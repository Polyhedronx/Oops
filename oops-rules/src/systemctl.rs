use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `systemctl` typos like `systemctrl`, `systemctl status` etc.
pub struct Systemctl;

impl Rule for Systemctl {
    fn name(&self) -> &'static str {
        "systemctl"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("systemc")
            && !command.script.starts_with("systemctl ")
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        // Known typos: systemctrl, systemclt, systmctl, etc.
        // Strip the "systemc" prefix and replace with "systemctl "
        let after_prefix = &command.script[7..]; // skip "systemc"
        // Skip the remaining typo chars until we hit a space or end
        let rest = after_prefix.find(' ')
            .map(|i| &after_prefix[i..])
            .unwrap_or("");
        vec![CorrectedCommand::new(
            format!("systemctl{}", rest),
            self.name(),
            self.priority(),
            Some("Fix 'systemc…' → 'systemctl'".into()),
        )]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemctrl() {
        let cmd = Command::new("systemctrl status nginx", None);
        assert!(Systemctl.match_command(&cmd));
        assert_eq!(
            Systemctl.get_new_command(&cmd)[0].script,
            "systemctl status nginx"
        );
    }

    #[test]
    fn test_correct_already() {
        let cmd = Command::new("systemctl status nginx", None);
        assert!(!Systemctl.match_command(&cmd));
    }
}
