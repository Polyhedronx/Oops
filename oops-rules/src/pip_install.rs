use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `pip install` with a mistyped package name by suggesting the closest
/// match from the error output.
pub struct PipInstall;

impl Rule for PipInstall {
    fn name(&self) -> &'static str {
        "pip_install"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("pip install ")
            && !command.script.starts_with("pip3 install ")
        {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("could not find a version")
                    || lower.contains("no matching distribution")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let parts: Vec<&str> = command.script.split_whitespace().collect();
        if parts.len() < 3 {
            return vec![];
        }
        let pkg = parts[2];

        // Try known common pip package typos
        let fixes: &[(&str, &str)] = &[
            ("requirments", "requirements"),
            ("requirment", "requirement"),
            ("virtualenv", "virtualenv"),
            ("django", "Django"),
            ("flask", "Flask"),
            ("request", "requests"),
            ("numpy", "numpy"),
            ("panda", "pandas"),
            ("pillow", "Pillow"),
            ("pyaml", "pyyaml"),
            ("bs4", "beautifulsoup4"),
        ];

        let mut results = Vec::new();
        for (typo, correct) in fixes {
            if pkg.eq_ignore_ascii_case(typo) {
                let corrected = format!(
                    "pip install {}",
                    if parts.len() > 3 {
                        format!("{} {}", correct, parts[3..].join(" "))
                    } else {
                        correct.to_string()
                    }
                );
                results.push(CorrectedCommand::new(
                    corrected.trim().to_string(),
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
    fn test_pip_typo_fix() {
        let cmd = Command::new(
            "pip install requirments",
            Some("Could not find a version that satisfies the requirement".into()),
        );
        assert!(PipInstall.match_command(&cmd));
        let r = PipInstall.get_new_command(&cmd);
        assert_eq!(r[0].script, "pip install requirements");
    }

    #[test]
    fn test_pip_not_matched() {
        let cmd = Command::new(
            "pip install requests",
            Some("Successfully installed".into()),
        );
        assert!(!PipInstall.match_command(&cmd));
    }
}
