use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `grep` is run on a directory without `-r`, suggest adding the
/// recursive flag.
pub struct GrepRecursive;

impl Rule for GrepRecursive {
    fn name(&self) -> &'static str {
        "grep_recursive"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("grep ") || command.script.contains(" -r") {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("is a directory")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        // Insert -r after existing flags (if any), before the pattern.
        let script = if let Some(rest) = command.script.strip_prefix("grep ") {
            let first_char = rest.chars().next().unwrap_or(' ');
            if first_char == '-' {
                // There are flags — insert -r after them
                if let Some(idx) = rest.find(' ') {
                    format!("grep {} -r{}", &rest[..idx], &rest[idx..])
                } else {
                    format!("grep {} -r", rest)
                }
            } else {
                format!("grep -r {}", rest)
            }
        } else {
            return vec![];
        };

        vec![CorrectedCommand::new(
            script,
            self.name(),
            self.priority(),
            Some("Add -r to search directories recursively".into()),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grep_directory() {
        let cmd = Command::new(
            "grep foo bar/",
            Some("grep: bar/: Is a directory".into()),
        );
        assert!(GrepRecursive.match_command(&cmd));
        let r = GrepRecursive.get_new_command(&cmd);
        assert_eq!(r[0].script, "grep -r foo bar/");
    }

    #[test]
    fn test_grep_with_flags() {
        let cmd = Command::new(
            "grep -n foo bar/",
            Some("grep: bar/: Is a directory".into()),
        );
        assert!(GrepRecursive.match_command(&cmd));
        let r = GrepRecursive.get_new_command(&cmd);
        assert_eq!(r[0].script, "grep -n -r foo bar/");
    }

    #[test]
    fn test_already_recursive() {
        let cmd = Command::new(
            "grep -r foo bar/",
            None,
        );
        assert!(!GrepRecursive.match_command(&cmd));
    }

    #[test]
    fn test_not_a_directory() {
        let cmd = Command::new(
            "grep foo file.txt",
            Some("".into()),
        );
        assert!(!GrepRecursive.match_command(&cmd));
    }
}
