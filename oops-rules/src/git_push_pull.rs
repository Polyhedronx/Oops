use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `git push` is rejected because the remote has new commits,
/// suggest pulling first and then pushing.
pub struct GitPushPull;

impl Rule for GitPushPull {
    fn name(&self) -> &'static str {
        "git_push_pull"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("git push") {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("rejected")
                    || lower.contains("failed to push")
                    || lower.contains("remote contains")
                    || lower.contains("updates were rejected")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let parts: Vec<&str> = command.script.split_whitespace().collect();
        // Extract remote and branch from "git push [remote] [branch]"
        let remote = parts.get(2).copied().unwrap_or("origin");
        let branch = parts.get(3).copied().unwrap_or("");

        let pull = if branch.is_empty() {
            format!("git pull {} && git push", remote)
        } else {
            format!("git pull {} {} && git push {} {}", remote, branch, remote, branch)
        };

        vec![CorrectedCommand::new(
            pull,
            self.name(),
            self.priority(),
            Some("Pull before pushing to avoid rejection".into()),
        )]
    }

    fn priority(&self) -> i32 {
        800
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_rejected() {
        let cmd = Command::new(
            "git push origin main",
            Some("! [rejected] main -> main (fetch first)".into()),
        );
        assert!(GitPushPull.match_command(&cmd));
        let r = GitPushPull.get_new_command(&cmd);
        assert_eq!(
            r[0].script,
            "git pull origin main && git push origin main"
        );
    }

    #[test]
    fn test_failed_to_push() {
        let cmd = Command::new(
            "git push",
            Some("Updates were rejected because the remote contains work".into()),
        );
        assert!(GitPushPull.match_command(&cmd));
        let r = GitPushPull.get_new_command(&cmd);
        assert_eq!(r[0].script, "git pull origin && git push");
    }

    #[test]
    fn test_not_git_push() {
        let cmd = Command::new("git pull", None);
        assert!(!GitPushPull.match_command(&cmd));
    }
}
