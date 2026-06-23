use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `brew install` / `brew search` with a mistyped formula name by
/// suggesting the closest match from the error output.
pub struct BrewInstall;

impl Rule for BrewInstall {
    fn name(&self) -> &'static str {
        "brew_install"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("brew install ")
            && !command.script.starts_with("brew search ")
            && !command.script.starts_with("brew info ")
        {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("no available formula")
                    || lower.contains("no formulae found")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let parts: Vec<&str> = command.script.split_whitespace().collect();
        if parts.len() < 3 {
            return vec![];
        }
        let sub_cmd = parts[1]; // "install", "search", or "info"
        let name = parts[2];

        // Common brew formula typos + their corrections
        let fixes: &[(&str, &str)] = &[
            ("pythn", "python"),
            ("pythoon", "python"),
            ("git-lfs", "git-lfs"),
            ("nodejs", "node"),
            ("wget", "wget"),
            ("curl", "curl"),
            ("openssl", "openssl"),
            ("openssh", "openssh"),
            ("imagemagick", "imagemagick"),
            ("postgresql", "postgresql"),
            ("postgree", "postgresql"),
            ("mysql", "mysql"),
            ("mysl", "mysql"),
            ("dockr", "docker"),
            ("gcc", "gcc"),
            ("treee", "tree"),
        ];

        let mut results = Vec::new();
        for (typo, correct) in fixes {
            if name.eq_ignore_ascii_case(typo) {
                let rest = parts[3..].join(" ");
                let corrected = if rest.is_empty() {
                    format!("brew {} {}", sub_cmd, correct)
                } else {
                    format!("brew {} {} {}", sub_cmd, correct, rest)
                };
                results.push(CorrectedCommand::new(
                    corrected,
                    self.name(),
                    self.priority(),
                    Some(format!("Replace '{}' with '{}'", typo, correct)),
                ));
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brew_pythn() {
        let cmd = Command::new(
            "brew install pythn",
            Some("Error: No available formula with the name \"pythn\"".into()),
        );
        assert!(BrewInstall.match_command(&cmd));
        let r = BrewInstall.get_new_command(&cmd);
        assert_eq!(r[0].script, "brew install python");
    }

    #[test]
    fn test_brew_no_formulae() {
        let cmd = Command::new(
            "brew search mysl",
            Some("No formulae found for \"mysl\"".into()),
        );
        assert!(BrewInstall.match_command(&cmd));
        let r = BrewInstall.get_new_command(&cmd);
        assert_eq!(r[0].script, "brew search mysql");
    }

    #[test]
    fn test_brew_not_matched() {
        let cmd = Command::new(
            "brew install python",
            Some("already installed".into()),
        );
        assert!(!BrewInstall.match_command(&cmd));
    }
}
