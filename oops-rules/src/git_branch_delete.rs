use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `git branch -d` fails because the branch is not fully merged,
/// suggest `git branch -D` to force-delete.
pub struct GitBranchDelete;

impl Rule for GitBranchDelete {
    fn name(&self) -> &'static str {
        "git_branch_delete"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("git branch -d ")
            || command.script.starts_with("git branch --delete ")
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        // Replace -d with -D (or --delete with --delete --force)
        let script = if let Some(rest) = command.script.strip_prefix("git branch -d ") {
            format!("git branch -D {}", rest)
        } else if let Some(rest) = command.script.strip_prefix("git branch --delete ") {
            format!("git branch --delete --force {}", rest)
        } else {
            return vec![];
        };

        vec![CorrectedCommand::new(
            script,
            self.name(),
            self.priority(),
            Some("Force-delete the branch with -D".into()),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_d_with_d() {
        let cmd = Command::new(
            "git branch -d old-feature",
            Some("not fully merged".into()),
        );
        assert!(GitBranchDelete.match_command(&cmd));
        let r = GitBranchDelete.get_new_command(&cmd);
        assert_eq!(r[0].script, "git branch -D old-feature");
    }

    #[test]
    fn test_long_flag() {
        let cmd = Command::new("git branch --delete fix", None);
        assert!(GitBranchDelete.match_command(&cmd));
        let r = GitBranchDelete.get_new_command(&cmd);
        assert_eq!(r[0].script, "git branch --delete --force fix");
    }

    #[test]
    fn test_no_match_uppercase_d() {
        let cmd = Command::new("git branch -D done", None);
        assert!(!GitBranchDelete.match_command(&cmd));
    }
}
