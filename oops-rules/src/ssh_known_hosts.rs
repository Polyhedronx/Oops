use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When SSH complains about a host key mismatch, suggest removing the
/// offending key from `known_hosts`.
pub struct SshKnownHosts;

impl Rule for SshKnownHosts {
    fn name(&self) -> &'static str {
        "ssh_known_hosts"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("ssh ") {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("remote host identification has changed")
                    || lower.contains("host key verification failed")
                    || lower.contains("offending key")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        // Extract hostname from the ssh command
        let parts: Vec<&str> = command.script.split_whitespace().collect();
        let host = parts.get(1).copied().unwrap_or("<host>");

        // Try to extract the offending key line number from error output
        let line_number = command
            .output
            .as_ref()
            .and_then(|out| {
                out.lines()
                    .find(|l| l.contains("known_hosts") && l.contains(':'))
                    .and_then(|l| l.split(':').nth(1))
                    .and_then(|s| s.trim().parse::<u32>().ok())
            });

        let remove_cmd = if let Some(line) = line_number {
            format!("sed -i '{}d' ~/.ssh/known_hosts && ssh {}", line, host)
        } else {
            format!(
                "ssh-keygen -R {} && ssh {}",
                host, host
            )
        };

        vec![CorrectedCommand::new(
            remove_cmd,
            self.name(),
            self.priority(),
            Some("Remove old host key and retry SSH".into()),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_key_changed() {
        let cmd = Command::new(
            "ssh admin@server.com",
            Some(
                "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!\n\
                 Offending key in /home/user/.ssh/known_hosts:42".into(),
            ),
        );
        assert!(SshKnownHosts.match_command(&cmd));
        let r = SshKnownHosts.get_new_command(&cmd);
        assert!(r[0].script.contains("known_hosts") && r[0].script.contains("ssh "));
    }

    #[test]
    fn test_not_ssh() {
        let cmd = Command::new(
            "scp file server:",
            Some("Host key verification failed".into()),
        );
        assert!(!SshKnownHosts.match_command(&cmd));
    }

    #[test]
    fn test_ssh_ok() {
        let cmd = Command::new("ssh user@host", Some("Permission denied".into()));
        assert!(!SshKnownHosts.match_command(&cmd));
    }
}
