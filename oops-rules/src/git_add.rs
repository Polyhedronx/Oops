use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `git commit` fails because files are unstaged, and `git commit --amend`
/// isn't the answer, suggest staging the files first.
pub struct GitAdd;

impl Rule for GitAdd {
    fn name(&self) -> &'static str {
        "git_add"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("git commit")
            && !command.script.starts_with("git diff")
        {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("did you forget to add")
                    || lower.contains("untracked files")
                    || lower.contains("changes not staged for commit")
                    || lower.contains("nothing added to commit")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        // If it's a commit, suggest add-then-commit
        if command.script.starts_with("git commit") {
            let rest: String = if command.script.contains(" -m ") {
                let msg = extract_commit_message(&command.script);
                if msg.is_empty() {
                    String::new()
                } else {
                    format!(" -m {}", msg)
                }
            } else {
                String::new()
            };

            vec![CorrectedCommand::new(
                format!("git add . && git commit{}", rest),
                self.name(),
                self.priority(),
                Some("Stage all files then commit".into()),
            )]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        900
    }
}

fn extract_commit_message(script: &str) -> String {
    if let Some(idx) = script.find(" -m ") {
        let after = &script[idx + 4..].trim();
        // Preserve the quoted message if present
        if let Some(inner) = after.strip_prefix('"') {
            if let Some(end) = inner.find('"') {
                format!("\"{}\"", &inner[..end])
            } else {
                after.to_string()
            }
        } else if let Some(inner) = after.strip_prefix('\'') {
            if let Some(end) = inner.find('\'') {
                format!("'{}'", &inner[..end])
            } else {
                after.to_string()
            }
        } else {
            after.split_whitespace().next().unwrap_or("").to_string()
        }
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forgot_to_add() {
        let cmd = Command::new(
            "git commit -m \"fix\"",
            Some(
                "Changes not staged for commit:\n\
                 modified: src/main.rs".into(),
            ),
        );
        assert!(GitAdd.match_command(&cmd));
        let r = GitAdd.get_new_command(&cmd);
        assert_eq!(r[0].script, "git add . && git commit -m \"fix\"");
    }

    #[test]
    fn test_untracked_files() {
        let cmd = Command::new(
            "git commit",
            Some(
                "Untracked files:\n  new_file.rs\n\
                 nothing added to commit but untracked files present".into(),
            ),
        );
        assert!(GitAdd.match_command(&cmd));
        assert_eq!(
            GitAdd.get_new_command(&cmd)[0].script,
            "git add . && git commit"
        );
    }

    #[test]
    fn test_not_commit() {
        let cmd = Command::new("git push", None);
        assert!(!GitAdd.match_command(&cmd));
    }
}
