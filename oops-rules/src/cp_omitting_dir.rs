use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `cp` complains about omitting a directory because `-r` isn't
/// specified, add the recursive flag.
pub struct CpOmittingDir;

impl Rule for CpOmittingDir {
    fn name(&self) -> &'static str {
        "cp_omitting_dir"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("cp ") || command.script.contains(" -r") {
            return false;
        }
        command.output.as_ref().is_some_and(|out| {
            let lower = out.to_lowercase();
            lower.contains("-r not specified")
                || lower.contains("omitting directory")
        })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let rest = command.script.strip_prefix("cp ").unwrap_or("");
        vec![CorrectedCommand::new(
            format!("cp -r {}", rest),
            self.name(),
            self.priority(),
            Some("Add -r to copy directories recursively".into()),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_dir_no_r() {
        let cmd = Command::new(
            "cp src /tmp/backup",
            Some("cp: -r not specified; omitting directory 'src'".into()),
        );
        assert!(CpOmittingDir.match_command(&cmd));
        assert_eq!(
            CpOmittingDir.get_new_command(&cmd)[0].script,
            "cp -r src /tmp/backup"
        );
    }

    #[test]
    fn test_already_recursive() {
        let cmd = Command::new("cp -r src /tmp", None);
        assert!(!CpOmittingDir.match_command(&cmd));
    }

    #[test]
    fn test_not_cp() {
        let cmd = Command::new("mv dir /tmp", None);
        assert!(!CpOmittingDir.match_command(&cmd));
    }
}
