use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "oops",
    version,
    about = "Correct your previous console command",
    after_help = "Quick start:\n  eval \"$(oops --alias)\"               # one-liner, works for bash/zsh\n  iex (oops --alias)                    # PowerShell\n  oops --install                        # permanent setup (writes to shell config)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Print shell alias (auto-detects bash/zsh/pwsh).
    /// Usage: eval "$(oops --alias)"  /  iex (oops --alias)
    #[arg(short, long, value_name = "ALIAS_NAME", num_args = 0..=1, default_missing_value = "oops")]
    pub alias: Option<String>,

    /// Run shell logger (captures output of all commands to FILE).
    #[arg(long, value_name = "FILE")]
    pub shell_logger: Option<String>,

    /// Run without confirmation (auto-select first correction).
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Repeat on failure (wrap corrected command to retry oops).
    #[arg(short = 'r', long)]
    pub repeat: bool,

    /// Enable debug output.
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Fix mode: correct the command passed after this flag.
    /// Used by shell aliases: oops --fix "$@"
    #[arg(long)]
    pub fix: bool,

    /// Path to oops binary (for alias generation).
    #[arg(long, value_name = "PATH")]
    pub bin: Option<String>,

    /// One-shot setup: write config + shell alias, auto-detect shell.
    #[arg(long)]
    pub install: bool,

    /// Force a specific command (prevents infinite recursion with --repeat).
    #[arg(long, value_name = "COMMAND")]
    pub force_command: Option<String>,

    /// The failed command (positional arguments).
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command_args: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate shell integration script for specified shell.
    #[command(name = "shell")]
    Shell {
        /// Shell name: bash, zsh, or powershell.
        #[arg(default_value = "bash")]
        shell: String,

        /// Enable experimental instant mode (PTY-based output capture).
        #[arg(long)]
        instant: bool,
    },

    /// List all available correction rules.
    #[command(name = "rules")]
    Rules,

    /// Print the current configuration.
    #[command(name = "config")]
    Config,

    /// Initialize a default configuration file.
    #[command(name = "init")]
    Init,
}
