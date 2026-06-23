use oops_core::config::Config;
use oops_core::corrector::Corrector;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::utils;

/// Run the main fix workflow.
///
/// Returns the selected command script, or `None` if no correction was found
/// or the user aborted.
pub fn run(config: &Config, raw_command: &[String]) -> Option<String> {
    // Build the Command object
    let command = if raw_command.is_empty() {
        utils::get_command_from_history()
            .map(|parts| utils::create_command(&parts, config))
            .unwrap_or_else(|| {
                eprintln!("Oops: No previous command found in shell history.");
                eprintln!();
                eprintln!("Shell integration may not be set up. Quick start:");
                eprintln!();
                eprintln!("  eval \"$(oops --alias)\"   # bash / zsh");
                eprintln!("  iex (oops --alias)        # PowerShell");
                eprintln!();
                eprintln!("Permanent setup:");
                eprintln!("  oops --install");
                std::process::exit(1);
            })
    } else {
        utils::create_command(raw_command, config)
    };

    if command.script.trim().is_empty() {
        eprintln!("Oops: Empty command, nothing to do.");
        return None;
    }

    // Get corrections
    let rules = oops_rules::get_all_rules();
    let corrector: Corrector<'_> = Corrector::new(rules, config.clone());
    let corrections = corrector.get_corrected_commands(&command);

    if corrections.is_empty() {
        eprintln!("Oops: No corrections found for '{}'", command.script);
        return None;
    }

    // If confirmation is not required, select the first one
    if !config.require_confirmation {
        let selected = &corrections[0];
        return Some(format_output(selected, config));
    }

    // Try TUI; fall back to auto-select if unavailable
    match oops_tui::run_tui(&corrections, Some(&command.script)) {
        Ok(Some(c)) => return Some(format_output(&c, config)),
        Ok(None) => {
            eprintln!("Oops: aborted.");
            return None;
        }
        Err(e) => {
            eprintln!("Oops: TUI unavailable ({}), auto-selecting.", e);
        }
    }

    // Fallback: list all corrections on stderr, auto-select first to stdout
    eprintln!("Oops: auto-selecting '{}' [{}]",
        corrections[0].script, corrections[0].rule_name);
    if corrections.len() > 1 {
        eprintln!("       (+{} more)", corrections.len() - 1);
        for (i, cmd) in corrections.iter().enumerate().skip(1) {
            eprintln!("       {}) {} [{}]", i + 1, cmd.script, cmd.rule_name);
        }
    }
    Some(format_output(&corrections[0], config))
}

/// Format the final output for the shell to eval.
fn format_output(correction: &CorrectedCommand, config: &Config) -> String {
    let script = &correction.script;

    if config.repeat {
        let alias = utils::get_alias();
        let debug_flag = if config.debug { " --debug" } else { "" };
        format!(
            "{} || {} {}--force-command '{}'",
            script, alias, debug_flag, script
        )
    } else {
        script.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oops_core::Config;

    #[test]
    fn test_format_output_simple() {
        let cmd = CorrectedCommand::new(
            "git push".into(),
            "test_rule",
            1000,
            None,
        );
        let config = Config::default();
        assert_eq!(format_output(&cmd, &config), "git push");
    }
}
