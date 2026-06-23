# 变更日志

Oops 项目的所有重要变更记录。

## [0.1.0] — 2026-06-23

### 新增 — 初始实现

#### 项目脚手架
- Cargo workspace，包含 5 个 crate：`oops-core`、`oops-shell`、`oops-rules`、`oops-tui`、`oops-cli`
- 统一 workspace 依赖（`serde`、`toml`、`clap`、`ratatui`、`crossterm`、`nucleo`、`similar` 等）

#### oops-core（核心库）
- `Command` 类型：表示失败的命令及其错误输出，支持惰性的 shell 感知脚本分词
- `Rule` trait：修正规则的核心抽象（`match_command`、`get_new_command`、`priority` 等）
- `CorrectedCommand` 类型：存储单个纠正建议（脚本、规则名、优先级、描述）
- `Corrector` 引擎：遍历已注册规则，按匹配+启用+需要输出进行过滤，按优先级去重/排序
- `Config`：基于 TOML 的配置，支持 serde 反序列化和环境变量覆盖（`OOPS_*` 前缀）
- 规则组合器：`AppOnly<T>`（限定规则作用于特定应用）、`SudoCapable<T>`（生成 sudo 变体）
- 工具函数：`is_app()`、`output_contains_any()`、`rerun_and_capture()`、`get_command_from_history()`
- `logger` 模块：Unix PTY shell 输出捕获守护进程（Instant Mode 支持）：
  - `libc::forkpty()` 创建 PTY + fork 子进程执行 `$SHELL`
  - `libc::poll()` I/O 多路复用（stdin → PTY → stdout + 日志文件）
  - 环形缓冲日志（2 MiB，包装时清除 512 KiB）
  - `cfmakeraw` raw 模式 + SIGWINCH 终端尺寸转发
  - Windows 上输出明确的不支持信息
- 13 个单元测试，覆盖 Command、Config、Corrector 和工具函数

#### oops-shell（Shell 集成库）
- `Shell` trait：定义 `app_alias()`、`instant_mode_alias()`、`split_command()`、`quote()`、`how_to_configure()`
- `Bash`：通过 `fc -ln -10` 生成完整 alias，支持配置检测（`.bashrc` / `.bash_profile`）
- `Zsh`：通过 `fc -ln -10` 生成 alias，配置路径 `.zshrc`，`print -s` 存入历史
- `PowerShell`：通过 `Get-History` 生成 alias，`$PROFILE` 配置
- `detect_shell()`：根据 `$SHELL` 或 `$PSModulePath` 环境变量自动检测
- 2 个 Shell 检测测试

#### oops-rules（规则库）— 19 条规则
| 规则 | 优先级 | 说明 |
|------|--------|------|
| `sudo` | 500 | 检测到"权限不足"时添加 `sudo` |
| `mkdir_p` | 800 | 父目录不存在时给 `mkdir` 加 `-p` |
| `cd_mkdir` | 850 | 目标不存在时先创建目录再 `cd` |
| `touch` | 850 | 父目录不存在时先 `mkdir -p` 再 `touch` |
| `python_execute` | 900 | 无执行权限的 `.py` 文件加 `python` 前缀 |
| `chmod_x` | 900 | 脚本无法执行时建议 `chmod +x` 或 `sudo` |
| `sl_ls` | 1000 | 修正 `sl` 打字错误为 `ls` |
| `rm_dir` | 1000 | 删除目录失败时加 `-rf` |
| `man_no_space` | 1000 | 修正子命令前缺少空格（如 `gitbranch` → `git branch`） |
| `apt_get` | 1000 | 修正常见 `apt-get` 打字错误（udpate → update 等） |
| `brew_unknown_command` | 1000 | 修正常见 `brew` 命令打字错误 |
| `cargo_no_command` | 1000 | 修正 `cargo` 子命令打字错误（buil → build 等） |
| `pip_unknown_command` | 1000 | 修正 `pip` 子命令打字错误（istall → install 等） |
| `npm_wrong_command` | 1000 | 修正 `npm` 子命令打字错误，含 Levenshtein 建议 |
| `docker_not_command` | 1000 | 修正 `docker` 子命令打字错误 |
| `git_not_command` | 1000 | 修正 `git` 子命令打字错误 + Levenshtein 建议 |
| `no_command` | 3000 | 兜底匹配"command not found"错误 |
| `no_such_file` | 3000 | 兜底匹配"No such file or directory"错误 |
| `unknown_command` | 5000 | 兜底匹配"unknown command"错误 |
- 22 个单元测试覆盖规则匹配和纠正逻辑

#### oops-tui（TUI 交互库）
- **ListPanel**：可滚动、可过滤的纠正列表，显示序号、命令和规则名，当前选中项反色高亮
- **PreviewPanel**：选中纠正的详情——规则名、优先级、描述、纠正后的命令，以及通过 `similar` crate 生成的字符级差异对比
- **InputBar**：模糊过滤的文本输入框，带光标和激活/非激活视觉状态
- **StatusBar**：按键提示，彩色标签（`↑↓/jk:导航`、`Enter:执行`、`Esc/q:退出`、`Tab:预览`、`/:过滤`）
- **Nucleo 模糊匹配**：实时 Unicode 感知模糊过滤（Helix 编辑器同款匹配器）
- **键盘快捷键**：`↑↓`/`jk` 导航，`Enter` 确认，`Esc`/`q`/`Ctrl+C` 取消，`Tab` 切换预览，`/` 进入过滤模式
- **终端缩放处理**：自动适�调整布局；终端小于 40×10 时显示提示信息
- **Raw 模式 + 备用屏幕**：通过 crossterm 实现完整的终端状态管理

#### oops-cli（命令行入口）
- 基于 `clap` derive 的 CLI，支持子命令和参数
- 子命令：`shell <bash|zsh|pwsh>`、`rules`、`config`、`init`
- `--alias [name]`：输出 shell alias 函数，供 `eval` 使用
- `--yes`：跳过 TUI，自动选择第一个纠正
- `--debug`：启用调试日志
- `--repeat`：包装输出以在失败时重试 oops
- `--force-command`：防止无限递归
- `--shell-logger <file>`：Shell 输出捕获守护进程，Unix PTY 实现，Windows 显示不支持
- `fix_command::run()`：编排 Command 创建 → Corrector 匹配 → TUI/自动选择流程
- 2 个单元测试

### 构建与测试状态
- ✅ Release 构建：0 警告、0 错误
- ✅ 全部测试套件：38 个测试通过（1 CLI + 13 核心 + 22 规则 + 2 Shell）
- ✅ Clippy：零警告（`cargo clippy -- -D warnings`）
- ✅ 二进制可运行：`--help`、`--alias`、`rules` 子命令已验证

### 2026-06-23 后续修复

#### Shell Logger 实现（#1）
- `oops-core/src/logger.rs`：新增 230+ 行 Unix PTY shell logger
- `oops-core/Cargo.toml`：添加 `libc` Unix-only 依赖
- `oops-cli/src/main.rs`：`run_shell_logger()` 从空壳 TODO 改为调用 `oops_core::logger::run_shell_logger()`

#### 开箱即用集成（#12）
- `--alias` 自动检测 shell（不再默认 bash），支持 `eval "$(oops --alias)"` 一行命令
- 新增 `--install` flag：等价 `oops init`，一键写入配置+alias
- 首次运行无历史时输出友好的安装指引（bash/zsh/pwsh 三行命令）
- `--help` 更新为开箱即用风格 quick start

#### 规则扩展 — 最终 6 条（#11）→ 达成 40 条 🎉
- 新增 6 条规则（34 → **40**）：
  | 规则 | 优先级 | 说明 |
  |------|--------|------|
  | `cp_omitting_dir` | 1000 | cp 目录忘加 `-r` |
  | `git_merge` | 1000 | merge 冲突 → `--abort` / `mergetool` |
  | `git_stash` | 800 | checkout 被本地修改挡住 → stash |
  | `git_rm_staged` | 1000 | rm 失败 → `--cached` 保留磁盘文件 |
  | `port_already_use` | 1000 | 端口占用 → kill 后重试 |
  | `composer_command` | 1000 | composer 子命令 typo → Levenshtein |

#### 规则扩展 — 通用规则（#10）
- 新增 3 条规则（31 → 34）：
  | 规则 | 优先级 | 说明 |
  |------|--------|------|
  | `git_add` | 900 | 忘记 `git add` → 建议 stage 再 commit |
  | `systemctl` | 1000 | `systemctrl` → `systemctl` |
  | `ssh_known_hosts` | 1000 | SSH host key 变更 → 删除旧 key 重试 |

#### 规则扩展 — 包管理规则（#9）
- 新增 4 条包管理规则（27 → 31）：
  | 规则 | 优先级 | 说明 |
  |------|--------|------|
  | `apt_get_search` | 1000 | `apt-get search` → `apt-cache search` |
  | `brew_install` | 1000 | `brew install` 公式名 typo 修正 |
  | `pip_install` | 1000 | `pip install` 包名 typo 修正 |
  | `npm_run_script` | 1000 | `npm run` 脚本名 typo → Levenshtein 建议 |

#### 规则扩展 — 系统命令规则（#8）
- 新增 3 条系统命令规则（24 → 27）：
  | 规则 | 优先级 | 说明 |
  |------|--------|------|
  | `cd_parent` | 1000 | `cd..` → `cd ..` |
  | `ls_all` | 1000 | `lsa`/`lsl`/`lsla` → `ls -a`/`-l`/`-la` |
  | `grep_recursive` | 1000 | 目录上 grep 忘加 `-r` |

#### 规则扩展 — Git 规则（#7）
- 新增 5 条 Git 规则（19 → 24）：
  | 规则 | 优先级 | 说明 |
  |------|--------|------|
  | `git_push_pull` | 800 | push 被拒时建议先 pull |
  | `git_pull_uncommitted` | 800 | 本地未提交时建议 stash + pull + pop |
  | `git_commit_amend` | 900 | "nothing added to commit" → `git add` 或 `--amend` |
  | `git_branch_delete` | 1000 | `-d` 失败 → 建议 `-D` 强制删除 |
  | `git_branch_exists` | 1000 | 分支已存在 → 建议 checkout |
- 新增 16 个测试（38 → 54）覆盖所有新规则

#### 文档更新 + P1 改进（#6）
- `DEVELOPMENT.md`：更新 Instant Mode 和 CI 状态为已完成，补充 `logger.rs` 到项目结构
- `oops shell --bin <PATH>`：手动指定二进制路径，通过 `OOPS_BIN_PATH` env var 传递
- `oops-shell/src/lib.rs`：新增 `resolve_bin_path()` 共享工具（检查 env var → `current_exe()` → `"oops"`）
- `oops init`：除 TOML 配置外，自动追加 shell alias 到 shell 配置文件（`.bashrc`/`.zshrc`/`$PROFILE`），检测已有配置避免重复

#### CI 基础设施（#5）
- 新增 `.github/workflows/ci.yml`：push/PR 自动触发
- 流程：`cargo check` → `cargo clippy`（零警告门禁） → `cargo test` → `cargo build --release`
- 运行在 `ubuntu-latest`（覆盖 Unix 代码路径，包括 PTY shell logger）

#### Instant Mode 完善（#4）
- CLI：`oops shell <name> --instant` 输出 instant mode alias（`oops-cli/src/cli.rs`、`main.rs`）
- Shell：`Bash`/`Zsh::instant_mode_alias()` 改为两阶段模式：
  - 首次调用（无 `OOPS_INSTANT_MODE`）：启动 `--shell-logger` PTY 守护进程
  - PTY 内调用（`OOPS_INSTANT_MODE` 已设）：注入不可见 PS1 标记（`\x1b]777;oops\x07`）用于命令边界检测
- 核心：`read_output_from_log()` — 从环形缓冲日志文件读取命令输出
  - 去除 ANSI CSI/OSC 转义序列
  - 启发式检测 shell 提示符以划定输出边界
  - Windows stub 返回 `None`（回退到 re-run）
- `create_command()` 在 instant mode 下优先使用日志输出，失败时回退到 `rerun_and_capture()`
- 辅助函数：`is_instant_mode()`、`instant_mode_log_path()`

#### 死代码清理（#3）
- 删除 `get_rules()` 永远返回空 Vec 的死函数（`oops-rules/src/lib.rs`）
- `oops-core/src/consts.rs`：删除 6 个未使用常量（`DEFAULT_PRIORITY`、`ALL_ENABLED`、`DEFAULT_ALIAS`、`DEFAULT_SHELL_LOGGER_LOG`、`ENV_PREFIX`、`ENV_SHELL_ALIASES`），保留 9 个
- 接线：`utils.rs` 和 `oops-shell/src/utils.rs` 中 `std::env::var()` 改用常量替代硬编码字符串（`ENV_HISTORY`、`ENV_ALIAS`、`ENV_SHELL`）

#### Side Effect 系统清理（#2）
- 移除 `SideEffectFn` 类型、`CorrectedCommand::side_effect` 字段、`with_side_effect()` 方法
- 移除 `Rule::side_effect()` trait 方法及 `AppOnly`/`SudoCapable` 转发
- 移除 `Corrector` 中空的侧效应连接块（类型不匹配，无法工作；无规则实现）

#### Clippy 全部清理
- `map_or(false, ...)` → `is_some_and(...)` — 20 处（oops-core 2、oops-rules 17、oops-tui 1）
- `map_or(true, ...)` → `is_none_or(...)` — 2 处（git_not_command、npm_wrong_command）
- `needless_return` → 删除 — 4 处（main.rs 子命令分支）
- `useless_format!` → `.to_string()` — 1 处（shell_trait.rs）
- `derivable_impls` → `#[derive(Default)]` — 1 处（config.rs RulesConfig）
- `needless_borrow` → 移除引用 — 1 处（corrector.rs）
- `collapsible_if` → 合并条件 — 2 处（git_not_command、npm_wrong_command）
- `op_ref` → 直接比较 — 1 处（npm_wrong_command）
