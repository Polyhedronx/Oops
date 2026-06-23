use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix Docker commands when a subcommand is not found.
pub struct DockerNotCommand;

impl Rule for DockerNotCommand {
    fn name(&self) -> &'static str {
        "docker_not_command"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("docker ")
            && command.output.as_ref().is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("is not a docker command")
                    || lower.contains("unknown command")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results = Vec::new();

        let fixes = [
            ("contianer", "container"),
            ("conatiner", "container"),
            ("iamge", "image"),
            ("iamges", "images"),
            ("pul", "pull"),
            ("psuh", "push"),
        ];

        for (typo, correct) in &fixes {
            if command.script.contains(typo) {
                results.push(CorrectedCommand::new(
                    command.script.replace(typo, correct),
                    self.name(),
                    1000,
                    Some(format!("Fix typo: {} -> {}", typo, correct)),
                ));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_typo() {
        let cmd = Command::new(
            "docker iamge ls",
            Some("docker: 'iamge' is not a docker command.".into()),
        );
        assert!(DockerNotCommand.match_command(&cmd));
        let results = DockerNotCommand.get_new_command(&cmd);
        assert_eq!(results[0].script, "docker image ls");
    }
}
