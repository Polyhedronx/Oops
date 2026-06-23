# Oops

**Oops** 帮你纠正输错的终端命令。它是 [thefuck](https://github.com/nvbn/thefuck) 的 Rust 重写版，使用 [ratatui](https://ratatui.rs/) 构建了 `fzf` 风格的交互式 TUI。

命令打错了？敲一下 `oops`，从候选纠正中选一个，回车就行。

```
$ git pish origin main
git: 'pish' is not a git command.

$ oops
┌──────────────────────────────────────┐
│ > git push origin main              [git_not_cmd]               │
│   git pull origin main              [git_pull]                  │
└──────────────────────────────────────┘
```

---

## 安装

### 前置

- Rust 1.80+（[rustup](https://rustup.rs/)）
- 终端：Windows Terminal（推荐）、WezTerm、Alacritty，或任意 Unix 终端

### 三行命令

```
git clone https://github.com/Polyhedronx/oops.git
cd oops
cargo build --release
```

然后根据你的 Shell 执行对应的安装命令：

```
# PowerShell
.\target\release\oops.exe --install
. $PROFILE

# bash / zsh
./target/release/oops --install
source ~/.bashrc   # 或 source ~/.zshrc
```


### 不想改配置文件？临时使用

```
# PowerShell
iex (.\target\release\oops.exe --alias | Out-String)

# bash / zsh
eval "$(./target/release/oops --alias)"
```

---

## 使用

### 基本用法

```bash
oops              # 纠正上一条失败的命令
oops git pish     # 纠正指定的命令
oops --yes        # 自动选第一条纠正，不启动 TUI（适合脚本/不喜欢交互）
oops --repeat     # 纠正后的命令如果还失败，包装一层自动重试
```

### TUI 操作

| 按键 | 功能 |
|------|------|
| `↑` `↓` / `j` `k` | 上下导航 |
| `Enter` | 选中并执行 |
| `/` | 输入过滤文字（模糊匹配） |
| `Tab` | 展开差异对比面板 |
| `Esc` / `q` / `Ctrl+C` | 退出，不执行任何纠正 |

列表支持实时模糊过滤——输入几个字母就能快速定位。匹配引擎基于 [nucleo](https://github.com/helix-editor/nucleo)（Helix 编辑器同款），支持 Unicode 和拼音首字母。

> Windows 用户：TUI 在 **Windows Terminal**、**WezTerm**、**Alacritty** 下完整可用。传统 `conhost`（cmd.exe）和 VS Code 内置终端会回退到自动选择第一条。

### 子命令一览

```bash
oops shell <bash|zsh|pwsh>    # 打印指定 Shell 的 alias
oops shell bash --instant      # 打印 Instant Mode alias（实验性，需 Unix PTY）
oops rules                     # 列出全部 40 条规则
oops config                    # 查看当前配置
oops init                      # 初始化配置 + shell alias
oops --install                 # 等效 oops init，无交互一键安装
```

---

## 配置

配置文件路径：`~/.config/oops/config.toml`。首次运行 `oops init` 或 `oops --install`
会自动生成。

```toml
# 不需要 TUI？设为 false
require_confirmation = true

# 重跑命令获取输出的超时时间（秒）
wait_command = 3
wait_slow_command = 15

# 失败时自动重试
repeat = false

# 将纠正后的命令写入 Shell 历史
alter_history = true

# 调试模式
debug = false

# 启用/排除的规则。可以使用 glob，如 "git_*"
rules = "all"
exclude_rules = []

# 手动覆盖特定规则的优先级（数值越小越靠前）
[priority_overrides]
sudo = 500
```

环境变量也可以覆盖：`OOPS_REQUIRE_CONFIRMATION`、`OOPS_DEBUG`、`OOPS_REPEAT`、
`OOPS_WAIT_COMMAND`、`OOPS_RULES`。

---

## 规则

Oops 内置 **40 条纠正规则**，覆盖 Git、系统命令、包管理器等常见场景：

| 类别 | 数量 | 代表规则 |
|------|------|---------|
| **Git** | 9 | `git_push_pull` push 被拒先 pull、`git_commit_amend` 忘 add→amend、`git_merge` 冲突→abort、`git_stash` checkout 前 stash、`git_branch_delete` -d→-D、`git_branch_exists` 分支已存在、`git_not_command` typo 建议… |
| **系统命令** | 12 | `sudo` 权限不足加 sudo、`mkdir_p` 缺 -p、`cd_parent` `cd..`→`cd ..`、`cp_omitting_dir` 缺 -r、`grep_recursive` 目录上缺 -r、`chmod_x` 无执行权限、`rm_dir` 缺 -rf、`ls_all` `lsa`→`ls -a`、`sl_ls` `sl`→`ls`、`systemctl` typo… |
| **包管理器** | 10 | `apt_get`/`apt_get_search`、`brew_unknown_command`/`brew_install`、`cargo_no_command`、`pip_unknown_command`/`pip_install`、`npm_wrong_command`/`npm_run_script`、`docker_not_command`、`composer_command` |
| **通用** | 9 | `man_no_space` 缺空格、`no_command`/`unknown_command`/`no_such_file` 兜底匹配、`python_execute` .py 无执行权限、`touch` 父目录不存在、`ssh_known_hosts` key 冲突、`port_already_use` 端口占用… |

完整规则列表：`oops rules`。

---

## Instant Mode

Instant Mode 通过后台 PTY 守护进程实时捕获终端输出，命令失败时无需重新执行即可
拿到错误信息。

仅 **Linux / macOS** 可用，需要 `oops shell bash --instant`（或 `zsh --instant`）。

```bash
$ eval "$(oops shell bash --instant)"
# 进入 PTY shell，此后命令的错误输出会被自动记录
# 敲 oops 时无需重跑命令即可拿到错误信息
```

详见 [DEVELOPMENT.md](docs/DEVELOPMENT.md#-p2-instant-mode--实时输出捕获)。

---

## 项目结构

```
oops-cli/     二进制入口 — CLI 参数解析 + fix_command 流程编排
oops-core/    核心库   — Command、Rule trait、Corrector、Config、logger
oops-rules/   规则库   — 40 条内置纠正规则
oops-tui/     TUI 库   — ratatui 交互式选择器（模糊过滤 + diff 预览）
oops-shell/   集成库   — Bash / Zsh / PowerShell alias 生成
```

---

## 开发

```bash
cargo check                   # 快速编译检查
cargo build --release         # Release 构建
cargo test                    # 运行全部 99 个测试
cargo clippy -- -D warnings   # Lint 检查

# 单独跑某个 crate 的测试
cargo test -p oops-rules
```

添加新规则 → 看 [DEVELOPMENT.md](docs/DEVELOPMENT.md#添加新规则)。

---

## 致谢

灵感来自 [thefuck](https://github.com/nvbn/thefuck)。依赖以下优秀项目：

[ratatui](https://ratatui.rs/) · [clap](https://docs.rs/clap/) · [nucleo](https://github.com/helix-editor/nucleo) · [similar](https://docs.rs/similar/) · [crossterm](https://docs.rs/crossterm/)

## 许可

MIT
