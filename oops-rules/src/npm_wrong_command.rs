use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `npm` commands when a wrong subcommand is used.
pub struct NpmWrongCommand;

impl Rule for NpmWrongCommand {
    fn name(&self) -> &'static str {
        "npm_wrong_command"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("npm ")
            && command.output.as_ref().is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("unknown command")
                    || lower.contains("did you mean")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results = Vec::new();

        let parts: Vec<&str> = command.script.split_whitespace().collect();
        if parts.len() >= 2 {
            use strsim::damerau_levenshtein;
            let known = [
                "install", "uninstall", "update", "start", "stop", "test",
                "run", "build", "publish", "init", "audit", "list", "ls",
                "config", "cache", "doctor", "help", "login", "logout",
                "outdated", "prune", "restart", "search", "shrinkwrap",
                "version", "view",
            ];

            let cmd = parts[1].to_lowercase();
            // Find the closest known npm subcommand
            let mut best: Option<(&str, usize)> = None;
            for known_cmd in &known {
                let dist = damerau_levenshtein(&cmd, known_cmd);
                if dist < cmd.len().min(known_cmd.len())
                    && best.is_none_or(|(_, d)| dist < d)
                {
                    best = Some((known_cmd, dist));
                }
            }

            if let Some((suggestion, _)) = best {
                if suggestion != cmd {
                    results.push(CorrectedCommand::new(
                        format!("npm {} {}", suggestion, parts[2..].join(" ")),
                        self.name(),
                        1000,
                        Some(format!("Replace '{}' with '{}'", cmd, suggestion)),
                    ));
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npm_typo() {
        let cmd = Command::new(
            "npm isntall express",
            Some("Unknown command: \"isntall\"".into()),
        );
        assert!(NpmWrongCommand.match_command(&cmd));
        let results = NpmWrongCommand.get_new_command(&cmd);
        assert_eq!(results[0].script.trim(), "npm install express");
    }
}
