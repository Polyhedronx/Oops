use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `git commit` reports "nothing added to commit", suggest either
/// `git add` + `git commit` or `git commit --amend` (when amend mode detected).
pub struct GitCommitAmend;

impl Rule for GitCommitAmend {
    fn name(&self) -> &'static str {
        "git_commit_amend"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("git commit") {
            return false;
        }
        // Don't intercept if user is already amending
        if command.script.contains("--amend") || command.script.contains("-a") {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("nothing added to commit")
                    || lower.contains("no changes added to commit")
                    || lower.contains("nothing to commit")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results = Vec::new();

        // Suggest adding all and committing
        let message = extract_message(&command.script);
        results.push(CorrectedCommand::new(
            format!(
                "git add . && git commit{}",
                if message.is_empty() {
                    String::new()
                } else {
                    format!(" -m {}", message)
                }
            ),
            self.name(),
            self.priority(),
            Some("Stage all changes and commit".into()),
        ));

        // Also suggest amending the previous commit
        results.push(CorrectedCommand::new(
            "git commit --amend --no-edit".to_string(),
            self.name(),
            self.priority() + 1, // slightly lower priority than add+commit
            Some("Amend the previous commit (reuse message)".into()),
        ));

        // If it was a `git commit -m "msg"` — suggest add + amend
        if !message.is_empty() {
            results.push(CorrectedCommand::new(
                format!(
                    "git add . && git commit --amend -m {}",
                    message
                ),
                self.name(),
                self.priority() + 2,
                Some("Stage all and amend with same message".into()),
            ));
        }

        results
    }

    fn priority(&self) -> i32 {
        900
    }
}

/// Extract the message from `git commit -m "msg"`.
fn extract_message(script: &str) -> String {
    if let Some(idx) = script.find("-m ") {
        let after_m = &script[idx + 3..].trim();
        // Handle quoted and unquoted messages
        if after_m.starts_with('"') {
            after_m
                .trim_matches('"')
                .to_string()
        } else if after_m.starts_with('\'') {
            after_m.trim_matches('\'').to_string()
        } else {
            after_m.split_whitespace().next().unwrap_or("").to_string()
        }
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nothing_to_commit() {
        let cmd = Command::new(
            "git commit -m \"fix bug\"",
            Some("nothing added to commit".into()),
        );
        assert!(GitCommitAmend.match_command(&cmd));
        let r = GitCommitAmend.get_new_command(&cmd);
        assert!(r.iter().any(|c| c.script.contains("git add")));
        assert!(r.iter().any(|c| c.script.contains("--amend")));
    }

    #[test]
    fn test_no_changes_added() {
        let cmd = Command::new(
            "git commit",
            Some("no changes added to commit".into()),
        );
        assert!(GitCommitAmend.match_command(&cmd));
    }

    #[test]
    fn test_no_match_already_amending() {
        let cmd = Command::new(
            "git commit --amend",
            Some("nothing added to commit".into()),
        );
        assert!(!GitCommitAmend.match_command(&cmd));
    }

    #[test]
    fn test_no_match_success() {
        let cmd = Command::new(
            "git commit -m ok",
            Some("1 file changed".into()),
        );
        assert!(!GitCommitAmend.match_command(&cmd));
    }
}
