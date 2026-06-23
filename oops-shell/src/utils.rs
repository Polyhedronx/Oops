use crate::bash::Bash;
use crate::powershell::PowerShell;
use crate::shell_trait::Shell;
use crate::zsh::Zsh;

/// Detect the current shell from environment variables.
pub fn detect_shell() -> Box<dyn Shell> {
    // 1. OOPS_SHELL — explicit override, always respected
    if let Ok(shell_name) = std::env::var(oops_core::consts::ENV_SHELL) {
        return match shell_name.to_lowercase().as_str() {
            "zsh" => Box::new(Zsh),
            "powershell" | "pwsh" | "powershell.exe" => Box::new(PowerShell),
            _ => Box::new(Bash),
        };
    }

    // 2. Windows: PSModulePath is the tell — ignore inherited SHELL from Git Bash
    #[cfg(windows)]
    {
        if std::env::var("PSModulePath").is_ok() {
            return Box::new(PowerShell);
        }
        // Check SHELL only on Windows when PSModulePath is absent (e.g. MSYS2 / Cygwin)
        if let Ok(sh) = std::env::var("SHELL") {
            let lower = sh.to_lowercase();
            if lower.contains("zsh") { return Box::new(Zsh); }
            if lower.contains("bash") { return Box::new(Bash); }
        }
        return Box::new(PowerShell);
    }

    // 3. Unix: use $SHELL
    #[cfg(not(windows))]
    {
        if let Ok(sh) = std::env::var("SHELL") {
            let lower = sh.to_lowercase();
            if lower.contains("zsh") { return Box::new(Zsh); }
            if lower.contains("bash") { return Box::new(Bash); }
        }
        Box::new(Bash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell_default() {
        let shell = detect_shell();
        let name = shell.name();
        assert!(name == "bash" || name == "zsh" || name == "powershell");
    }

    #[test]
    fn test_detect_shell_from_env() {
        let shell_name = oops_core::consts::ENV_SHELL;
        unsafe { std::env::set_var(shell_name, "zsh") };
        let shell = detect_shell();
        assert_eq!(shell.name(), "zsh");
        unsafe { std::env::remove_var(shell_name) };
    }
}
