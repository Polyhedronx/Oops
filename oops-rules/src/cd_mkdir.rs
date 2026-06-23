use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When `cd` fails because the directory doesn't exist, suggest `mkdir -p` first.
pub struct CdMkdir;

impl Rule for CdMkdir {
    fn name(&self) -> &'static str {
        "cd_mkdir"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("cd ")
            && command.output.as_ref().is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("no such file or directory")
                    || lower.contains("not a directory")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        if let Some(dir) = command.script.strip_prefix("cd ") {
            let dir = dir.trim();
            vec![CorrectedCommand::new(
                format!("mkdir -p {} && cd {}", dir, dir),
                self.name(),
                850,
                Some("Create the directory before changing into it".into()),
            )]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        850
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cd_mkdir_match() {
        let cmd = Command::new(
            "cd /nonexistent/dir",
            Some("cd: no such file or directory: /nonexistent/dir".into()),
        );
        assert!(CdMkdir.match_command(&cmd));
        let results = CdMkdir.get_new_command(&cmd);
        assert!(results[0].script.contains("mkdir -p"));
        assert!(results[0].script.contains("&& cd"));
    }
}
