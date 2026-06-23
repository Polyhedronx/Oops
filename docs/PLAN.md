# 实现计划

## 背景

用 Rust 重写 thefuck 命令行纠错工具，使用 ratatui 构建 fzf 风格的交互式 TUI 前端。项目名 **oops**，跨平台（Windows/Linux/macOS），支持 Bash/Zsh/PowerShell。

## 架构概览

```
oops/                              # Cargo workspace
├── oops-cli/     (二进制)         # CLI 入口、fix_command 流程、shell_logger
├── oops-core/    (库)             # Command、Rule trait、Corrector、Config
├── oops-rules/   (库)             # 所有内置规则的 struct + 注册表
├── oops-tui/     (库)             # ratatui 交互 UI（列表+过滤+预览）
└── oops-shell/   (库)             # Shell trait、Bash/Zsh/PowerShell 实现
```

依赖方向（无循环）：
```
oops-cli → oops-core, oops-rules, oops-tui, oops-shell
oops-rules → oops-core
oops-tui → oops-core
oops-core → (独立)
oops-shell → (独立)
```

## 核心抽象

### Command（`oops-core/src/command.rs`）
```rust
pub struct Command {
    pub script: String,              // 原始命令："git pish"
    pub output: Option<String>,      // 错误输出（stdout+stderr）
    script_parts: OnceLock<Vec<String>>, // Shell 感知分词（惰性计算）
}
```

### Rule trait（`oops-core/src/rule.rs`）
```rust
pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;
    fn match_command(&self, command: &Command) -> bool;
    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand>;
    fn requires_output(&self) -> bool { true }
    fn priority(&self) -> i32 { 1000 }
    fn enabled_by_default(&self) -> bool { true }
    fn side_effect(&self, _old: &Command, _new: &str) -> Option<String> { None }
}
```

### Corrector 引擎
- 遍历所有已注册 `Rule`，过滤 `is_enabled && is_match`
- 收集所有 `CorrectedCommand`，按 script 去重（保留低优先级）
- 按 priority 升序排列
- 返回排序、去重后的纠正列表

## 规则系统

**编译进二进制**：每个规则是一个实现了 `Rule` trait 的 Rust struct，通过静态注册表汇总。

```rust
static REGISTRY: Lazy<Vec<Box<dyn Rule>>> = Lazy::new(|| {
    vec![
        Box::new(sudo::Sudo),
        Box::new(mkdir_p::MkdirP),
        Box::new(git_not_command::GitNotCommand),
        // ...
    ]
});
```

添加新规则只需两步：1) 创建 struct + `impl Rule`，2) 在注册表中添加 `Box::new(...)`。

**组合器**（替代 Python 的 `@git_support`/`@sudo_support`）：
- `AppOnly<T>` — 仅在 `script_parts[0]` 为指定应用名时匹配
- `SudoCapable<T>` — 额外生成带 `sudo` 前缀的纠正版本
- 辅助函数：`is_app()`、`output_contains_any()`

### 已实现规则（19 条）

| 优先级 | 规则 | 分类 |
|--------|------|------|
| 500 | `sudo` | 系统 |
| 800 | `mkdir_p` | 系统 |
| 850 | `cd_mkdir`、`touch` | 系统 |
| 900 | `python_execute`、`chmod_x` | 系统 |
| 1000 | `sl_ls`、`rm_dir`、`man_no_space` | 系统 |
| 1000 | `apt_get`、`brew_unknown_command`、`cargo_no_command`、`pip_unknown_command`、`npm_wrong_command`、`docker_not_command` | 包管理器 |
| 1000 | `git_not_command` | Git |
| 3000-5000 | `no_command`、`unknown_command`、`no_such_file` | 兜底匹配 |

### 计划中的规则（Phase 2）

- Git：`checkout`、`add`、`push`、`pull`、`commit_add`、`commit_amend`、`merge`、`stash`、`branch_delete`、`branch_exists`、`push_pull`、`pull_uncommitted`、`rm_staged`
- 系统：`cd_correction`、`cd_parent`、`quotation_marks`、`grep_recursive`、`ls_all`、`ls_lah`
- 包管理：`apt_get_search`、`pip_install`、`npm_run_script`、`brew_install`
- 通用：`systemctl`、`port_already_in_use`、`ssh_known_hosts`

## Shell 集成

### Shell trait
```rust
pub trait Shell: Send + Sync {
    fn name(&self) -> &'static str;
    fn app_alias(&self, alias_name: &str) -> String;
    fn instant_mode_alias(&self, alias_name: &str) -> String;
    fn split_command(&self, script: &str) -> Vec<String>;
    fn quote(&self, s: &str) -> String;
    fn how_to_configure(&self) -> ShellConfiguration;
}
```

### Alias 生成（Bash 示例）
生成的 Shell 函数流程：
1. 通过 `fc -ln -10` 导出 `TF_HISTORY`
2. 调用 `oops --fix "$@"`
3. `eval` 返回的纠正命令

### 命令输出捕获
- **重运行模式**（默认）：重新执行失败命令并捕获输出，可配置超时
- **Shell Logger 模式**：后台守护进程 `oops --shell-logger <file>` 捕获所有输出
- **即时模式**：修改 PS1 注入零宽标记，实现即时检测

## TUI 架构（ratatui）

### 布局
```
┌──────────────────────────────────────────────────────────┐
│  主布局（水平分割）                                        │
│  ┌─────────────────────────┐ ┌──────────────────────────┐│
│  │  ListPanel（左 55%）    │ │  PreviewPanel（右 45%）  ││
│  │  - 可滚动列表            │ │  - 命令差异对比           ││
│  │  - 高亮当前选中项        │ │  - 规则描述              ││
│  │  - 规则名 + 命令         │ │  - 可滚动                ││
│  └─────────────────────────┘ └──────────────────────────┘│
├──────────────────────────────────────────────────────────┤
│  InputBar：搜索过滤文本 [光标闪烁]                         │
├──────────────────────────────────────────────────────────┤
│  StatusBar：↑↓ 导航  Enter 执行  Esc/q 退出  Tab 预览    │
└──────────────────────────────────────────────────────────┘
```

### 状态机
```rust
pub struct TuiState {
    pub all_corrections: Vec<CorrectedCommand>,
    pub filtered_indices: Vec<usize>,
    pub selected_index: usize,
    pub filter_text: String,
    pub mode: AppMode,           // Selecting | Filtering | Confirmed | Aborted
    pub show_preview: bool,
}
```

### 核心交互
- **⬆️⬇️ / jk / Ctrl+N+P**：导航列表
- **输入文字**：实时模糊过滤（nucleo 引擎）
- **Enter**：执行选中的纠正
- **Tab**：切换预览面板
- **Esc**：清空过滤器 / 退出
- **Ctrl+C / q**：取消
- **--yes 模式**：跳过 TUI，直接输出第一个纠正到 stdout

### 关键依赖
- `ratatui` 0.29 — TUI 框架
- `crossterm` 0.28 — 跨平台终端 I/O
- `nucleo` 0.5 — 模糊匹配（Helix 编辑器同款）
- `similar` 2 — 差异展示（预览面板）

## 配置（TOML）

默认路径：`~/.config/oops/config.toml`

```toml
rules = "all"                    # "all" 或 ["git_*", "sudo"]
exclude_rules = []
[priority_overrides]
sudo = 500
wait_command = 3
wait_slow_command = 15
require_confirmation = true
repeat = false
alter_history = true
debug = false
```

加载优先级：编译默认值 → TOML 文件 → `OOPS_*` 环境变量 → CLI 参数

## CLI（clap derive）

```
oops                          # 纠正上一条命令
oops <command...>             # 纠正指定命令
oops --yes                    # 自动执行，无需确认
oops --alias [name]           # 输出 shell alias 脚本
oops shell <bash|zsh|pwsh>    # 生成指定 shell 集成脚本
oops --shell-logger <file>    # 启动 shell logger 守护进程
oops init                     # 生成默认配置文件
oops rules                    # 列出所有规则
oops config                   # 打印当前有效配置
```

## 实现阶段

### Phase 1：核心管线 ✅
- 搭建 Cargo workspace（`oops-core`、`oops-cli`、`oops-rules`、`oops-shell`）
- 实现 `Command`、`Rule` trait、`Config`（TOML 加载）
- 实现 `Corrector` 匹配引擎（match → collect → dedup → sort）
- 实现 Rerun 输出捕获
- 编写 5-10 条简单规则
- 实现基础 CLI（`--alias`、`--yes`、位置参数）
- 实现 Bash shell alias 生成

### Phase 2：Shell 集成 ✅
- 完善 Bash / Zsh / PowerShell 的 Shell trait 实现
- Shell 自动检测
- Instant mode 和 Shell Logger 桩实现
- `how_to_configure()` 安装指引输出

### Phase 3：规则扩展 ✅
- 19 条核心规则及测试
- 实现组合器（`AppOnly<T>`、`SudoCapable<T>`）
- 辅助函数库（`is_app()`、`output_contains_any()` 等）

### Phase 4：TUI（`oops-tui`） ✅
- 集成 ratatui + crossterm
- 实现 ListPanel（可滚动、高亮、规则名显示）
- 实现 InputBar（单行输入 + 光标）
- 实现 PreviewPanel（差异展示 + 规则描述）
- 集成 nucleo 模糊过滤
- 键盘快捷键系统

### Phase 5：打磨 ⏳
- [x] TOML 配置 + `OOPS_*` 环境变量覆盖
- [x] `oops init` / `oops rules` / `oops config` 子命令
- [x] Repeat 模式、alter_history 支持
- [ ] 完整即时模式实现
- [ ] Shell logger 守护进程
- [ ] CI/CD（GitHub Actions：ubuntu、macos、windows）
- [ ] Cargo crate 发布

## 验证方式

1. **单元测试**：每条规则的 `match_command` 和 `get_new_command` 编写参数化测试
2. **集成测试**：端到端测试（构造 Command → Corrector → 验证纠正列表）
3. **手动测试**：在 Bash/Zsh/PowerShell 中实际安装 alias，输入错误命令后运行 `oops`
4. **TUI 测试**：手动验证键盘导航、过滤、预览、确认/取消等交互

## 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 规则加载 | 编译进二进制 | 类型安全、编译期检查、零运行时开销 |
| 规则抽象 | Trait（非函数指针） | 支持组合包装、可携带状态 |
| 模糊匹配 | nucleo | Helix 同款，亚毫秒级 Unicode 模糊匹配 |
| 项目结构 | Workspace（5 crate） | 独立编译、并行构建 |
| 配置格式 | TOML | Rust 生态标准、serde 反序列化 |
| 终端后端 | crossterm | Windows/Linux/macOS 全平台、纯 Rust |
