use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `apt-get search` → `apt-cache search` (search is part of apt-cache,
/// not apt-get).
pub struct AptGetSearch;

impl Rule for AptGetSearch {
    fn name(&self) -> &'static str {
        "apt_get_search"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("apt-get search ")
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let rest = command.script.strip_prefix("apt-get search ").unwrap_or("");
        vec![CorrectedCommand::new(
            format!("apt-cache search {}", rest),
            self.name(),
            self.priority(),
            Some("Use 'apt-cache search' instead of 'apt-get search'".into()),
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
    fn test_apt_get_search() {
        let cmd = Command::new("apt-get search vim", None);
        assert!(AptGetSearch.match_command(&cmd));
        assert_eq!(
            AptGetSearch.get_new_command(&cmd)[0].script,
            "apt-cache search vim"
        );
    }

    #[test]
    fn test_not_search() {
        let cmd = Command::new("apt-get install vim", None);
        assert!(!AptGetSearch.match_command(&cmd));
    }
}
