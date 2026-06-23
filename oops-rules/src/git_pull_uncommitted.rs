use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `git pull` fails because of uncommitted local changes,
/// suggest stashing first.
pub struct GitPullUncommitted;

impl Rule for GitPullUncommitted {
    fn name(&self) -> &'static str {
        "git_pull_uncommitted"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("git pull") {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("overwritten by merge")
                    || lower.contains("commit your changes or stash them")
                    || lower.contains("please commit your changes")
                    || lower.contains("would be overwritten")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let pull_cmd = command.script.trim();
        vec![CorrectedCommand::new(
            format!(
                "git stash && {} && git stash pop",
                pull_cmd
            ),
            self.name(),
            self.priority(),
            Some(
                "Stash uncommitted changes, pull, then re-apply".into(),
            ),
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
    fn test_pull_overwrite() {
        let cmd = Command::new(
            "git pull origin main",
            Some("error: Your local changes would be overwritten by merge.".into()),
        );
        assert!(GitPullUncommitted.match_command(&cmd));
        let r = GitPullUncommitted.get_new_command(&cmd);
        assert!(r[0].script.contains("git stash"));
        assert!(r[0].script.ends_with("git stash pop"));
    }

    #[test]
    fn test_stash_them_message() {
        let cmd = Command::new(
            "git pull",
            Some("Please commit your changes or stash them before you merge.".into()),
        );
        assert!(GitPullUncommitted.match_command(&cmd));
    }

    #[test]
    fn test_no_match_clean_pull() {
        let cmd = Command::new("git pull", Some("Already up to date.".into()));
        assert!(!GitPullUncommitted.match_command(&cmd));
    }
}
