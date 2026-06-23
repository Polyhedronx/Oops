use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When creating a branch that already exists, suggest checking out
/// the existing branch instead.
pub struct GitBranchExists;

impl Rule for GitBranchExists {
    fn name(&self) -> &'static str {
        "git_branch_exists"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !(command.script.starts_with("git branch ")
            || command.script.starts_with("git checkout -b "))
        {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("already exists")
                    || lower.contains("fatal: a branch named")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let parts: Vec<&str> = command.script.split_whitespace().collect();

        if command.script.starts_with("git checkout -b ") && parts.len() >= 4 {
            // "git checkout -b <branch>" → "git checkout <branch>"
            let branch = parts[3];
            return vec![CorrectedCommand::new(
                format!("git checkout {}", branch),
                self.name(),
                self.priority(),
                Some(format!("Branch '{}' already exists", branch)),
            )];
        }

        // "git branch <name>" → just checkout it
        if parts.len() >= 2 {
            let branch = parts[2];
            return vec![CorrectedCommand::new(
                format!("git checkout {}", branch),
                self.name(),
                self.priority(),
                Some(format!("Branch '{}' already exists", branch)),
            )];
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkout_new_existing() {
        let cmd = Command::new(
            "git checkout -b feature",
            Some("fatal: a branch named 'feature' already exists.".into()),
        );
        assert!(GitBranchExists.match_command(&cmd));
        let r = GitBranchExists.get_new_command(&cmd);
        assert_eq!(r[0].script, "git checkout feature");
    }

    #[test]
    fn test_branch_create_existing() {
        let cmd = Command::new(
            "git branch feat",
            Some("fatal: A branch named 'feat' already exists.".into()),
        );
        assert!(GitBranchExists.match_command(&cmd));
    }

    #[test]
    fn test_no_match_created() {
        let cmd = Command::new(
            "git checkout -b new-branch",
            Some("Switched to a new branch 'new-branch'".into()),
        );
        assert!(!GitBranchExists.match_command(&cmd));
    }
}
