pub mod bash;
pub mod powershell;
pub mod shell_trait;
pub mod utils;
pub mod zsh;

pub use bash::Bash;
pub use powershell::PowerShell;
pub use shell_trait::{Shell, ShellConfiguration};
pub use utils::detect_shell;
pub use zsh::Zsh;

/// Resolve the binary path for use in generated shell aliases.
///
/// Checks, in order:
/// 1. `OOPS_BIN_PATH` env var (set by `--bin` flag)
/// 2. `std::env::current_exe()` (absolute path to running binary)
/// 3. Fallback to `"oops"` (rely on PATH lookup)
pub fn resolve_bin_path() -> String {
    std::env::var("OOPS_BIN_PATH").ok().or_else(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
    }).unwrap_or_else(|| "oops".to_string())
}
