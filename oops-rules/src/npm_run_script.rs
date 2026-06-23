use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `npm run` when the script name is misspelled — suggest similar scripts
/// from the error output.
pub struct NpmRunScript;

impl Rule for NpmRunScript {
    fn name(&self) -> &'static str {
        "npm_run_script"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("npm run ") {
            return false;
        }
        command
            .output
            .as_ref()
            .is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("missing script")
                    || lower.contains("did you mean")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results = Vec::new();
        let parts: Vec<&str> = command.script.split_whitespace().collect();
        let script_name = parts.get(2).copied().unwrap_or("");

        // Try common npm script typos
        let common_scripts: &[(&str, &str)] = &[
            ("dev", "dev"),
            ("build", "build"),
            ("test", "test"),
            ("start", "start"),
            ("lint", "lint"),
            ("format", "format"),
            ("deploy", "deploy"),
        ];

        // Use Levenshtein to find the closest match
        use strsim::damerau_levenshtein;
        let mut best: Option<(&str, usize)> = None;
        for (cmd, _) in common_scripts {
            let dist = damerau_levenshtein(script_name, cmd);
            if dist <= 2 && dist < script_name.len().max(cmd.len()).max(1)
                && best.is_none_or(|(_, d)| dist < d)
            {
                best = Some((*cmd, dist));
            }
        }

        if let Some((suggestion, _)) = best {
            if suggestion != script_name {
                let rest = if parts.len() > 3 {
                    format!(" {}", parts[3..].join(" "))
                } else {
                    String::new()
                };
                results.push(CorrectedCommand::new(
                    format!("npm run {}{}", suggestion, rest),
                    self.name(),
                    self.priority(),
                    Some(format!("Did you mean 'npm run {}'?", suggestion)),
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
    fn test_npm_missing_script() {
        let cmd = Command::new(
            "npm run devv",
            Some("Missing script: \"devv\"\nDid you mean this?\n  npm run dev".into()),
        );
        assert!(NpmRunScript.match_command(&cmd));
        let r = NpmRunScript.get_new_command(&cmd);
        assert_eq!(r[0].script, "npm run dev");
    }

    #[test]
    fn test_npm_not_missing() {
        let cmd = Command::new(
            "npm run build",
            Some("Build completed".into()),
        );
        assert!(!NpmRunScript.match_command(&cmd));
    }
}
