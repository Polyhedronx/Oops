use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `git rm` fails because the file has staged changes, suggest using
/// `--cached` or `-f` (force).
pub struct GitRmStaged;

impl Rule for GitRmStaged {
    fn name(&self) -> &'static str {
        "git_rm_staged"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("git rm ") {
            return false;
        }
        command.script.contains(" -f")
            || command.output.as_ref().is_some_and(|out| {
                out.to_lowercase().contains("has changes staged in the index")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let rest = command
            .script
            .strip_prefix("git rm ")
            .unwrap_or("")
            .replace("-f ", "")
            .replace(" -f", "");

        let mut results = Vec::new();
        // Suggest removing just from the index (keep on disk)
        results.push(CorrectedCommand::new(
            format!("git rm --cached {}", rest.trim()),
            self.name(),
            self.priority(),
            Some("Remove from index only, keep file on disk".into()),
        ));

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_staged() {
        let cmd = Command::new(
            "git rm file.txt",
            Some(
                "error: the following file has changes staged in the index:\n    file.txt".into(),
            ),
        );
        assert!(GitRmStaged.match_command(&cmd));
        let r = GitRmStaged.get_new_command(&cmd);
        assert_eq!(r[0].script, "git rm --cached file.txt");
    }

    #[test]
    fn test_rm_already_forced() {
        let cmd = Command::new("git rm -f file.txt", None);
        assert!(GitRmStaged.match_command(&cmd));
        let r = GitRmStaged.get_new_command(&cmd);
        assert_eq!(r[0].script, "git rm --cached file.txt");
    }

    #[test]
    fn test_rm_ok() {
        let cmd = Command::new("git rm old.txt", Some("rm 'old.txt'".into()));
        assert!(!GitRmStaged.match_command(&cmd));
    }
}
