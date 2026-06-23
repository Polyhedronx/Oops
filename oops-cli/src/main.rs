mod cli;
mod fix_command;

use clap::Parser;
use cli::{Cli, Commands};
use oops_core::config::Config;

fn main() {
    let cli = Cli::parse();

    // Load config
    let mut config = Config::load();

    // Apply CLI overrides
    if cli.yes {
        config.require_confirmation = false;
    }
    if cli.repeat {
        config.repeat = true;
    }
    if cli.debug {
        config.debug = true;
    }
    // Let shell implementations use this path instead of current_exe().
    if let Some(ref bin_path) = cli.bin {
        unsafe { std::env::set_var("OOPS_BIN_PATH", bin_path) };
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Shell { shell, instant }) => {
            if instant {
                print_instant_mode_alias(&shell, cli.alias.as_deref());
            } else {
                print_shell_alias(&shell, cli.alias.as_deref());
            }
        }
        Some(Commands::Rules) => {
            list_rules();
        }
        Some(Commands::Config) => {
            print_config(&config);
        }
        Some(Commands::Init) => {
            init_config();
        }
        None => {
            // Main fix mode
            if let Some(log_file) = cli.shell_logger {
                run_shell_logger(&log_file);
                return;
            }

            // --alias mode: auto-detect shell, print alias, exit
            if cli.alias.is_some() {
                print_alias_for_current_shell(cli.alias.as_deref());
                return;
            }

            // --install mode: one-shot setup
            if cli.install {
                init_config();
                return;
            }

            // Main fix workflow
            let raw_command: Vec<String> = if let Some(force_cmd) = cli.force_command {
                vec![force_cmd]
            } else {
                cli.command_args
            };

            match fix_command::run(&config, &raw_command) {
                Some(output) => {
                    // If OOPS_OUTPUT_FILE is set, write result there (for shell aliases
                    // that need to keep stdin as a terminal for TUI interaction)
                    if let Ok(path) = std::env::var("OOPS_OUTPUT_FILE") {
                        let _ = std::fs::write(&path, &output);
                    } else {
                        print!("{}", output);
                    }
                }
                None => {
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Auto-detect the current shell and print its alias.
fn print_alias_for_current_shell(alias_name: Option<&str>) {
    let shell = oops_shell::detect_shell();
    let alias = alias_name.unwrap_or("oops");
    print!("{}", shell.app_alias(alias));
}

fn print_shell_alias(shell_name: &str, alias_name: Option<&str>) {
    print_alias(shell_name, alias_name, false);
}

fn print_instant_mode_alias(shell_name: &str, alias_name: Option<&str>) {
    print_alias(shell_name, alias_name, true);
}

fn print_alias(shell_name: &str, alias_name: Option<&str>, instant: bool) {
    use oops_shell::{Bash, PowerShell, Shell, Zsh};

    let alias = alias_name.unwrap_or("oops");
    let shell: Box<dyn Shell> = match shell_name.to_lowercase().as_str() {
        "zsh" => Box::new(Zsh),
        "powershell" | "pwsh" => Box::new(PowerShell),
        _ => Box::new(Bash),
    };

    if instant {
        print!("{}", shell.instant_mode_alias(alias));
    } else {
        print!("{}", shell.app_alias(alias));
    }
}

fn list_rules() {
    let rules = oops_rules::get_all_rules();
    println!("Available rules ({} total):", rules.len());
    println!("{}", "-".repeat(40));
    for rule in rules {
        println!(
            "  {:30}  priority={:4}  default={}",
            rule.name(),
            rule.priority(),
            rule.enabled_by_default()
        );
    }
}

fn print_config(config: &Config) {
    println!("Current configuration:");
    println!("  require_confirmation: {}", config.require_confirmation);
    println!("  wait_command: {}s", config.wait_command);
    println!("  wait_slow_command: {}s", config.wait_slow_command);
    println!("  repeat: {}", config.repeat);
    println!("  alter_history: {}", config.alter_history);
    println!("  debug: {}", config.debug);
    println!("  history_limit: {}", config.history_limit);
    println!("  slow_commands: {:?}", config.slow_commands);
    if !config.exclude_rules.is_empty() {
        println!("  exclude_rules: {:?}", config.exclude_rules);
    }
}

fn init_config() {
    let config = Config::default_config();
    let config_path = Config::default_path()
        .unwrap_or_else(|| {
            eprintln!("Oops: Cannot determine config directory.");
            std::process::exit(1);
        });

    // Write TOML config
    let toml_str = toml::to_string_pretty(&config).unwrap_or_else(|e| {
        eprintln!("Oops: Cannot serialize config: {}", e);
        std::process::exit(1);
    });

    if !config_path.exists() {
        // Create parent directory
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!("Oops: Cannot create config directory: {}", e);
                std::process::exit(1);
            });
        }
        std::fs::write(&config_path, &toml_str).unwrap_or_else(|e| {
            eprintln!("Oops: Cannot write config: {}", e);
            std::process::exit(1);
        });
        println!("Config written to: {}", config_path.display());
    } else {
        println!("Config already exists at: {}", config_path.display());
    }

    // Write shell alias
    setup_shell_alias();
}

/// Detect the current shell and append the oops alias to its config file.
fn setup_shell_alias() {
    let shell = oops_shell::detect_shell();
    let shell_cfg = shell.how_to_configure();

    // Use the one-liner from how_to_configure() — NOT the expanded function body.
    // The one-liner calls `oops --alias` dynamically, so it survives binary moves.
    let one_liner = shell_cfg.content.trim();

    let config_path = expand_tilde(&shell_cfg.path);

    // Check if alias is already configured
    if config_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&config_path) {
            if contents.contains("oops --alias") || contents.contains("oops --fix") {
                println!(
                    "Shell alias already configured in {}",
                    shell_cfg.path
                );
                return;
            }
        }
    }

    // Append alias to shell config
    let entry = format!(
        "\n# oops — correct mistyped shell commands\n{}\n",
        one_liner
    );

    // Ensure parent directory exists (e.g. ~/Documents/PowerShell/)
    if let Some(parent) = config_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
    {
        Ok(mut f) => {
            use std::io::Write;
            if let Err(e) = writeln!(f, "{}", entry) {
                eprintln!("Oops: Failed to write to {}: {}", shell_cfg.path, e);
                print_manual_alias_instructions(&shell_cfg, one_liner);
                return;
            }
            println!("Shell alias written to: {}", shell_cfg.path);
            println!("Run '{}' to activate it now.", shell_cfg.reload);
        }
        Err(e) => {
            eprintln!("Oops: Cannot open {}: {}", shell_cfg.path, e);
            print_manual_alias_instructions(&shell_cfg, one_liner);
        }
    }
}

fn print_manual_alias_instructions(
    cfg: &oops_shell::ShellConfiguration,
    alias: &str,
) {
    println!();
    println!("Add the following to {}:", cfg.path);
    println!("───────────────");
    println!("{}", alias.trim());
    println!("───────────────");
    println!("Then run: {}", cfg.reload);
}

/// Expand a leading ~ to the user's home directory, and resolve `$PROFILE`
/// to PowerShell's actual profile path.
fn expand_tilde(path: &str) -> std::path::PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    if path == "$PROFILE" {
        if let Some(docs) = dirs::document_dir() {
            // Standard PowerShell profile path
            return docs.join("PowerShell").join("Microsoft.PowerShell_profile.ps1");
        }
    }
    std::path::PathBuf::from(path)
}

/// Shell logger mode: continuously capture shell output.
fn run_shell_logger(log_file: &str) {
    oops_core::logger::run_shell_logger(std::path::Path::new(log_file));
}
