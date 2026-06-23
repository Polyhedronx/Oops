# 开发指南

## 项目结构

```
Oops/
├── Cargo.toml                    # Workspace 根文件 — 共享版本号与依赖
├── docs/                         # 文档
│   ├── CHANGELOG.md              # 变更日志
│   ├── README.md                 # 项目概览
│   ├── PLAN.md                   # 实现计划
│   └── DEVELOPMENT.md            # 开发指南（当前文件）
├── oops-core/src/                # 核心抽象层（库）
│   ├── lib.rs                    # 公开 API 导出
│   ├── command.rs                # Command 结构体 + 脚本分词
│   ├── rule.rs                   # Rule trait + 组合器 + 辅助函数
│   ├── corrected_command.rs      # CorrectedCommand 结构体
│   ├── corrector.rs              # 匹配引擎（遍历→匹配→收集→去重→排序）
│   ├── config.rs                 # TOML 配置加载 + 环境变量覆盖
│   ├── consts.rs                 # 常量（优先级、超时、环境变量名）
│   ├── logger.rs                 # PTY shell logger（Instant Mode）
│   └── utils.rs                  # 重运行捕获、历史解析、别名辅助
├── oops-shell/src/               # Shell 集成层（库）
│   ├── lib.rs                    # 公开导出
│   ├── shell_trait.rs            # Shell trait 定义
│   ├── bash.rs                   # Bash 实现
│   ├── zsh.rs                    # Zsh 实现
│   ├── powershell.rs             # PowerShell 实现
│   └── utils.rs                  # Shell 检测
├── oops-rules/src/               # 内置规则（库）
│   ├── lib.rs                    # 模块声明 + REGISTRY 全局注册表
│   ├── sudo.rs                   # 添加 sudo
│   ├── mkdir_p.rs                # mkdir -p
│   ├── cd_mkdir.rs               # mkdir + cd
│   ├── chmod_x.rs                # chmod +x
│   ├── rm_dir.rs                 # rm -rf
│   ├── touch.rs                  # mkdir -p + touch
│   ├── sl_ls.rs                  # sl → ls
│   ├── man_no_space.rs           # 单词间插入空格
│   ├── python_execute.rs         # python script.py
│   ├── no_command.rs             # "command not found" 兜底
│   ├── unknown_command.rs        # "unknown command" 兜底
│   ├── no_such_file.rs           # "No such file" 兜底
│   ├── apt_get.rs                # apt-get 打字错误
│   ├── brew_unknown_command.rs   # brew 打字错误
│   ├── cargo_no_command.rs       # cargo 打字错误
│   ├── pip_unknown_command.rs    # pip 打字错误
│   ├── npm_wrong_command.rs      # npm 打字错误+建议
│   ├── docker_not_command.rs     # docker 打字错误
│   ├── git_not_command.rs        # git 打字错误+建议
│   ├── git_branch_delete.rs      # git branch -d → -D
│   ├── git_branch_exists.rs      # 分支已存在
│   ├── git_commit_amend.rs       # 忘记 git add → amend
│   ├── git_pull_uncommitted.rs   # stash + pull + pop
│   └── git_push_pull.rs          # push 被拒先 pull
├── oops-tui/src/                 # ratatui 交互界面（库）
│   ├── lib.rs                    # 公开 API：run_tui()
│   ├── app.rs                    # 事件循环、终端设置、渲染
│   ├── state.rs                  # TuiState、AppMode、模糊过滤逻辑
│   ├── event.rs                  # 按键映射、Action 枚举
│   └── components/
│       ├── mod.rs
│       ├── list_panel.rs         # 可滚动纠正列表
│       ├── preview_panel.rs      # 差异对比/预览面板
│       ├── input_bar.rs          # 过滤文本输入
│       └── status_bar.rs         # 按键提示栏
└── oops-cli/src/                 # CLI 二进制入口
    ├── main.rs                   # 参数分发、子命令处理
    ├── cli.rs                    # clap derive 结构体
    └── fix_command.rs            # 核心纠正流程编排
```

## 端到端流程

```
用户：$ git pish
bash：git: 'pish' is not a git command.
用户：$ oops
  │
  ▼
main.rs → Cli::parse()
  │
  ▼
fix_command::run()
  ├── 1. 从历史记录（fc -ln -10）或 CLI 参数获取失败的命令
  ├── 2. 创建 Command { script: "git pish", output: "git: 'pish' is not..." }
  ├── 3. Corrector::get_corrected_commands(command)
  │       ├── 遍历 REGISTRY 中的所有规则
  │       ├── 检查 is_rule_enabled(?)
  │       ├── 检查 requires_output(?) → 无输出则跳过
  │       ├── 调用 rule.match_command(command)
  │       └── 调用 rule.get_new_command(command) → Vec<CorrectedCommand>
  ├── 4. 去重 + 按优先级排序
  ├── 5. 如果 require_confirmation=true → 启动 TUI
  │       └── 用户选择纠正 → 返回 CorrectedCommand
  │   如果 require_confirmation=false → 使用第一条纠正
  └── 6. 向 stdout 输出纠正后的命令
bash：eval 输出 → 执行 "git push"
```

## 添加新规则

### 第一步：创建规则模块

在 `oops-rules/src/` 下创建 `your_rule.rs`：

```rust
use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

pub struct YourRule;

impl Rule for YourRule {
    fn name(&self) -> &'static str {
        "your_rule"  // 唯一名称，使用 snake_case
    }

    fn match_command(&self, command: &Command) -> bool {
        // 当此规则适用时返回 true。
        // 可用的数据：command.script、command.output、command.script_parts()
        command.script.starts_with("your_app ")
            && command.output.as_ref().map_or(false, |out| {
                out.to_lowercase().contains("error message")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        vec![CorrectedCommand::new(
            format!("corrected {}", command.script),
            self.name(),
            self.priority(),
            Some("人类可读的纠�描述".into()),
        )]
    }

    fn priority(&self) -> i32 {
        1000  // 数值越小越靠前，需要时覆盖此值
    }

    fn requires_output(&self) -> bool {
        true  // 如果匹配不需要输出则设为 false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match() {
        let cmd = Command::new("your_app fail", Some("error message".into()));
        assert!(YourRule.match_command(&cmd));
        let results = YourRule.get_new_command(&cmd);
        assert_eq!(results[0].script, "corrected your_app fail");
    }
}
```

### 第二步：注册规则

在 `oops-rules/src/lib.rs` 中：

1. 在文件顶部添加 `pub mod your_rule;`
2. 在 `REGISTRY` 的 `vec![]` 内添加 `Box::new(your_rule::YourRule),`

### 第三步：测试

```bash
cargo test -p oops-rules
```

### 使用规则组合器

```rust
// 无需手动编写应用名检查，使用组合器即可：

// 在注册表中：
Box::new(AppOnly::new(your_rule::YourRule, &["git", "hub"])),

// 或自动生成 sudo 变体：
Box::new(SudoCapable::new(your_rule::YourRule)),
```

### 辅助函数

```rust
use oops_core::rule::{is_app, output_contains_any};

fn match_command(&self, command: &Command) -> bool {
    is_app(command, &["git"])  // script_parts[0] == "git"
        && output_contains_any(command, &["permission denied", "eacces"])
}
```

## 构建与测试

```bash
# 检查编译（所有 crate）
cargo check

# Debug 构建
cargo build

# Release 构建
cargo build --release

# 运行全部测试
cargo test

# 运行指定 crate 的测试
cargo test -p oops-core
cargo test -p oops-rules
cargo test -p oops-shell

# 带输出运行测试
cargo test -- --nocapture

# 运行单个测试
cargo test -p oops-rules git_not_command::tests::test_git_pish

# 代码检查
cargo clippy -- -D warnings
```

## 依赖关系图

```
oops-cli (二进制)
  依赖：oops-core、oops-shell、oops-rules、oops-tui
  额外依赖：clap、toml、color-eyre、tracing

oops-core (库)
  独立 — 无内部依赖
  依赖：serde、toml、once_cell、regex、dirs、strsim、tracing

oops-rules (库)
  依赖：oops-core
  额外依赖：regex、once_cell、strsim

oops-tui (库)
  依赖：oops-core
  额外依赖：ratatui、crossterm、nucleo、similar

oops-shell (库)
  依赖：oops-core
  额外依赖：dirs、tracing
```

## 关键设计模式

### Rule Trait 优于函数指针

使用 trait 而非 `fn` 指针对，因为：
- 支持组合包装（`AppOnly<T>`、`SudoCapable<T>`）通过泛型
- 规则可以携带状态（缓存数据、预编译正则）
- 提供默认方法实现（`requires_output()`、`priority()` 等）

### Corrector 借用规则

`Corrector<'_>` 通过 `&[Box<dyn Rule>]` 借用规则，而非持有所有权。
这使得全局 `REGISTRY` 静态变量可以被共享，无需 clone 或 `Arc`。

### 静态注册表

规则注册在 `once_cell::sync::Lazy<Vec<Box<dyn Rule>>>` 中：
- 首次访问时初始化一次（线程安全）
- 通过 `oops_rules::get_all_rules()` 全局可访问
- 添加新规则只需一行代码

### TUI 状态机

```
Selecting ──输入字符──→ Filtering ──Esc──→ Selecting（清空过滤）
    │                       │
    │ Enter                 │ Enter
    ▼                       ▼
 Confirmed ───────────→ 退出并返回选中项

 Selecting ──Esc/q/Ctrl+C──→ Aborted ──→ 退出返回 None
```

## 配置加载顺序

1. **编译时默认值** — `oops_core::Config::default()`
2. **TOML 文件** — `~/.config/oops/config.toml`（如果存在）
3. **环境变量** — `OOPS_` 前缀：
   - `OOPS_REQUIRE_CONFIRMATION` → `true`/`false`
   - `OOPS_DEBUG` → `true`/`false`
   - `OOPS_REPEAT` → `true`/`false`
   - `OOPS_ALTER_HISTORY` → `true`/`false`
   - `OOPS_WAIT_COMMAND` → 秒数（整数）
   - `OOPS_RULES` → 逗号分隔的列表
4. **CLI 参数** — `--yes`、`--repeat`、`--debug`

## 已知问题与改进计划

以下功能为让 PowerShell/Windows 环境先跑通而暂时关闭或简化。每个条目记录了现状、原因和改进方案。

---

### ✅ [P0] rerun_and_capture — 命令输出捕获

**状态**：已修复。

**实现**：
1. Windows 上添加 `CREATE_NO_WINDOW`（0x08000000）标志，防止控制台窗口闪烁
2. 使用独立 watcher 线程 + `mpsc::channel` 实现超时控制，子进程留在主线程以支持 kill
3. `create_command` 已恢复调用 `rerun_and_capture`
4. 超时由 `wait_command`/`wait_slow_command` 配置控制

**涉及文件**：`oops-core/src/utils.rs`

---

### ✅ [P0] ratatui TUI — 交互式模糊选择器

**状态**：已修复。

**实现**：
1. `is_tui_capable()` 检测终端兼容性：
   - Windows：检查 `WT_SESSION`（Windows Terminal）、`TERM_PROGRAM`（WezTerm/Alacritty）
   - 排除 VS Code（`VSCODE_INJECTION`）和 JetBrains（`TERMINAL_EMULATOR`）
   - Unix：检查 `TERM` 是否含 `256color`/`kitty`/`alacritty`
2. 兼容终端 → 启动 ratatui TUI（模糊过滤 + diff 预览）
3. 不兼容终端 → 返回 `Err`，`fix_command.rs` 自动回退到列表 + 自动选择第一条

**待改进**：
- crossterm 0.29+ 对 Windows conhost 兼容性有改善，可升级测试

**涉及文件**：`oops-tui/src/lib.rs`、`oops-cli/src/fix_command.rs`

---

### [P1] 纯文本交互选择器 — 已移除

**现状**：`oops-cli/src/fix_command.rs` 中的 `text_select()` 函数已被删除。
原实现向 stderr 输出带编号的纠正列表，从 stdin 读取用户选择。

**原因**：当 shell alias 捕获 stdout 时（`$result = & oops --fix @args`），
`io::stdin().read_line()` 在 PowerShell 中可能阻塞。由于 TUI 是更好的交互方案，
暂不修复此路径。

**改进方案**：
1. TUI 修复后此 fallback 不再需要
2. 如需保留，改为从 `/dev/tty`（Unix）或 `CONIN$`（Windows）读取而非 stdin

**涉及文件**：`oops-cli/src/fix_command.rs`

---

### ✅ [P1] Shell alias — 开箱即用集成

**状态**：已完善。

**体验**：
```bash
eval "$(oops --alias)"    # bash/zsh — 一行命令即可使用
iex (oops --alias)         # PowerShell
oops --install             # 永久安装（写入 shell 配置文件）
```
- `--alias` 自动检测 shell 类型（不再默认 bash）
- 首次无配置运行时显示友好安装指引
- `oops init` 同时写入 TOML 配置 + shell alias
### ✅ [P2] 规则扩展 — 已达 40 条

**状态**：完成。从 19 条扩展到 40 条，覆盖 Git（+8）、系统命令（+6）、包管理（+4）、通用（+3）。

**新增规则清单**：
- Git：`git_add`, `git_push_pull`, `git_pull_uncommitted`, `git_commit_amend`, `git_merge`, `git_stash`, `git_rm_staged`, `git_branch_delete`, `git_branch_exists`（共 9 条，原有 1 条）
- 系统：`cp_omitting_dir`, `cd_parent`, `ls_all`, `grep_recursive`, `systemctl`, `ssh_known_hosts`
- 包管理：`apt_get_search`, `brew_install`, `pip_install`, `npm_run_script`, `composer_command`
- 通用：`port_already_use`

**涉及文件**：`oops-rules/src/*.rs`

---

### ✅ [P2] Instant mode — 实时输出捕获

**状态**：已实现。

**实现**：
1. `oops-core/src/logger.rs`：Unix PTY shell logger 守护进程（`libc::forkpty` + 环形缓冲日志）
2. `oops-shell/src/{bash,zsh}.rs`：`instant_mode_alias()` 两阶段模式
   - 首次调用：启动 `--shell-logger` PTY 守护进程
   - PTY 内部调用：注入不可见 PS1 标记（`\x1b]777;oops\x07`）用于命令边界检测
3. `oops-core/src/utils.rs`：`read_output_from_log()` 从日志读取命令输出，去除 ANSI 转义序列
4. `create_command()` 在 instant mode 下优先读日志，失败时回退到 `rerun_and_capture()`
5. CLI：`oops shell <name> --instant` 输出 instant mode alias

**涉及文件**：`oops-core/src/logger.rs`、`oops-core/src/utils.rs`、`oops-shell/src/{bash,zsh}.rs`、`oops-cli/src/{cli,main}.rs`

---

### [P3] CI/CD 与测试

**现状**：基础 CI 已实现。38 个单元测试覆盖核心逻辑，无集成测试。

**已实现**：
- `.github/workflows/ci.yml`：push/PR 触发，`ubuntu-latest` 运行 check → clippy → test → build

**待改进**：
1. CI 多平台矩阵：加 `macos-latest`、`windows-latest`
2. 集成测试：模拟 bash/zsh/pwsh 环境运行 `oops` 并验证输出
3. 添加 rustfmt 检查

**涉及文件**：`.github/workflows/ci.yml`

---

### [P3] crates.io 发布与打包

**现状**：项目为 workspace，各 crate 未单独发布。

**改进方案**：
1. 各 crate 添加 `description`、`documentation`、`repository` 等元数据
2. `cargo publish` 各 crate
3. Homebrew formula、AUR package、Windows Winget manifest
