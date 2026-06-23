use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When a port is already in use, suggest finding and killing the process
/// occupying it.
pub struct PortAlreadyUse;

impl Rule for PortAlreadyUse {
    fn name(&self) -> &'static str {
        "port_already_use"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.output.as_ref().is_some_and(|out| {
            let lower = out.to_lowercase();
            lower.contains("address already in use")
                || lower.contains("port is already in use")
                || lower.contains("eaddrinuse")
        })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        // Try to extract the port from common error messages:
        //   "address already in use :::3000"
        //   "port 8000 is already in use"
        //   "listen EADDRINUSE: address already in use :::8080"
        let port = command
            .output
            .as_ref()
            .and_then(|out| {
                // Search for a port number in the error message
                for word in out.split(|c: char| !c.is_alphanumeric()) {
                    if let Ok(p) = word.parse::<u16>() {
                        if p > 0 {
                            return Some(p);
                        }
                    }
                }
                None
            });

        let mut results = Vec::new();
        let original = command.script.trim();

        if let Some(p) = port {
            // Linux/macOS: lsof-based kill
            results.push(CorrectedCommand::new(
                format!("kill $(lsof -ti :{}) && {}", p, original),
                self.name(),
                self.priority(),
                Some(format!("Kill process on port {} and retry", p)),
            ));
            // Alternative: fuser
            results.push(CorrectedCommand::new(
                format!("fuser -k {}/tcp && {}", p, original),
                self.name(),
                self.priority() + 1,
                Some(format!("Kill process on port {} with fuser", p)),
            ));
        } else {
            results.push(CorrectedCommand::new(
                format!("lsof -ti :<PORT> | xargs kill && {}", original),
                self.name(),
                self.priority(),
                Some("Find and kill process on the conflicting port".into()),
            ));
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eaddrinuse() {
        let cmd = Command::new(
            "npm start",
            Some("Error: listen EADDRINUSE: address already in use :::3000".into()),
        );
        assert!(PortAlreadyUse.match_command(&cmd));
        let r = PortAlreadyUse.get_new_command(&cmd);
        assert!(r.iter().any(|c| c.script.contains("3000")));
        assert!(r.iter().any(|c| c.script.contains("npm start")));
    }

    #[test]
    fn test_port_in_use() {
        let cmd = Command::new(
            "python -m http.server 8080",
            Some("socket.error: [Errno 98] Address already in use".into()),
        );
        assert!(PortAlreadyUse.match_command(&cmd));
    }

    #[test]
    fn test_not_port_error() {
        let cmd = Command::new(
            "curl localhost",
            Some("Connection refused".into()),
        );
        assert!(!PortAlreadyUse.match_command(&cmd));
    }
}
