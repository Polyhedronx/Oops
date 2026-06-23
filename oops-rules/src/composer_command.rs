use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `composer` subcommand typos using Levenshtein-based suggestions.
pub struct ComposerCommand;

impl Rule for ComposerCommand {
    fn name(&self) -> &'static str {
        "composer_command"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("composer ") {
            return false;
        }
        // Match even without output — typo detection from script alone
        let parts: Vec<&str> = command.script.split_whitespace().collect();
        if parts.len() < 2 {
            return false;
        }
        let known: &[&str] = &[
            "install", "update", "require", "remove", "dump-autoload",
            "create-project", "init", "search", "show", "outdated",
            "validate", "status", "self-update", "config", "exec",
            "run-script", "check-platform-reqs", "fund", "licenses",
            "prohibits", "reinstall", "suggests", "depends", "why",
            "why-not", "archive", "browse", "clear-cache", "diagnose",
            "global", "help", "home", "list",
        ];
        !known.contains(&parts[1])
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let parts: Vec<&str> = command.script.split_whitespace().collect();
        if parts.len() < 2 {
            return vec![];
        }

        let cmd = parts[1];
        let known: &[&str] = &[
            "install", "update", "require", "remove", "dump-autoload",
            "create-project", "init", "search", "show", "outdated",
            "validate", "status", "self-update", "config", "exec",
            "run-script", "check-platform-reqs", "fund", "licenses",
            "prohibits", "reinstall", "suggests", "depends", "why",
            "why-not", "archive", "browse", "clear-cache", "diagnose",
            "global", "help", "home", "list",
        ];

        use strsim::damerau_levenshtein;
        let mut best: Option<(&str, usize)> = None;
        for &known_cmd in known {
            let dist = damerau_levenshtein(cmd, known_cmd);
            if dist <= 2 && dist < cmd.len().max(known_cmd.len()).max(1)
                && best.is_none_or(|(_, d)| dist < d)
            {
                best = Some((known_cmd, dist));
            }
        }

        if let Some((suggestion, _)) = best {
            if suggestion != cmd {
                let rest = parts[2..].join(" ");
                let script = if rest.is_empty() {
                    format!("composer {}", suggestion)
                } else {
                    format!("composer {} {}", suggestion, rest)
                };
                return vec![CorrectedCommand::new(
                    script,
                    self.name(),
                    self.priority(),
                    Some(format!("Did you mean 'composer {}'?", suggestion)),
                )];
            }
        }

        vec![]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composer_istall() {
        let cmd = Command::new("composer istall", None);
        assert!(ComposerCommand.match_command(&cmd));
        let r = ComposerCommand.get_new_command(&cmd);
        assert_eq!(r[0].script, "composer install");
    }

    #[test]
    fn test_composer_updtae() {
        let cmd = Command::new("composer updtae", None);
        assert!(ComposerCommand.match_command(&cmd));
        let r = ComposerCommand.get_new_command(&cmd);
        assert_eq!(r[0].script, "composer update");
    }

    #[test]
    fn test_composer_valid() {
        let cmd = Command::new("composer install", None);
        assert!(!ComposerCommand.match_command(&cmd));
    }
}
