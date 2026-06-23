use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `touch` with missing directories in the path.
pub struct Touch;

impl Rule for Touch {
    fn name(&self) -> &'static str {
        "touch"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("touch ")
            && command.output.as_ref().is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("no such file or directory")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        // Suggest mkdir -p on the parent directory first, then touch
        if let Some(path) = command.script.strip_prefix("touch ") {
            let path = path.trim();
            if let Some(parent) = std::path::Path::new(path).parent() {
                if parent != std::path::Path::new("") {
                    let parent_str = parent.display().to_string();
                    return vec![CorrectedCommand::new(
                        format!("mkdir -p {} && touch {}", parent_str, path),
                        self.name(),
                        850,
                        Some("Create parent directories before touching the file".into()),
                    )];
                }
            }
        }
        vec![]
    }

    fn priority(&self) -> i32 {
        850
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_match() {
        let cmd = Command::new(
            "touch /nonexistent/dir/file.txt",
            Some("touch: cannot touch '/nonexistent/dir/file.txt': No such file or directory".into()),
        );
        assert!(Touch.match_command(&cmd));
        let results = Touch.get_new_command(&cmd);
        assert!(results[0].script.contains("mkdir -p"));
    }

    #[test]
    fn test_touch_no_match() {
        let cmd = Command::new("touch file.txt", Some("".into()));
        assert!(!Touch.match_command(&cmd));
    }
}
