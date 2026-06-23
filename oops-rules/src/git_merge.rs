use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `git merge` produces conflicts, suggest aborting or using mergetool.
pub struct GitMerge;

impl Rule for GitMerge {
    fn name(&self) -> &'static str {
        "git_merge"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("git merge") {
            return false;
        }
        command.output.as_ref().is_some_and(|out| {
            let lower = out.to_lowercase();
            (lower.contains("conflict") || lower.contains("automatic merge failed"))
                && !lower.contains("merge --abort")
        })
    }

    fn get_new_command(&self, _command: &Command) -> Vec<CorrectedCommand> {
        vec![
            CorrectedCommand::new(
                "git merge --abort".into(),
                self.name(),
                self.priority(),
                Some("Abort the merge and return to pre-merge state".into()),
            ),
            CorrectedCommand::new(
                "git mergetool".into(),
                self.name(),
                self.priority() + 1,
                Some("Open merge tool to resolve conflicts".into()),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_conflict() {
        let cmd = Command::new(
            "git merge feature",
            Some(
                "Auto-merging src/main.rs\n\
                 CONFLICT (content): Merge conflict in src/main.rs\n\
                 Automatic merge failed; fix conflicts and then commit.".into(),
            ),
        );
        assert!(GitMerge.match_command(&cmd));
        let r = GitMerge.get_new_command(&cmd);
        assert!(r.iter().any(|c| c.script == "git merge --abort"));
        assert!(r.iter().any(|c| c.script == "git mergetool"));
    }

    #[test]
    fn test_merge_success() {
        let cmd = Command::new("git merge feature", Some("Already up to date.".into()));
        assert!(!GitMerge.match_command(&cmd));
    }

    #[test]
    fn test_not_merge() {
        let cmd = Command::new("git pull", Some("CONFLICT".into()));
        assert!(!GitMerge.match_command(&cmd));
    }
}
