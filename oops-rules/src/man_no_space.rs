use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix missing space before subcommand (e.g., `gitbranch` -> `git branch`).
pub struct ManNoSpace;

impl Rule for ManNoSpace {
    fn name(&self) -> &'static str {
        "man_no_space"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.split_whitespace().count() == 1
            && command.script.chars().any(|c| c.is_uppercase())
            && command.output.as_ref().is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("not found") || lower.contains("no manual entry")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        // Try to split camelCase into two words
        let script = &command.script;
        let mut results = Vec::new();

        // Find the first uppercase letter boundary
        for (i, c) in script.chars().enumerate().skip(1) {
            if c.is_uppercase() {
                let first = &script[..i];
                let second = &script[i..];
                let corrected = format!("{} {}", first.to_lowercase(), second);
                results.push(CorrectedCommand::new(
                    corrected,
                    self.name(),
                    1000,
                    Some("Insert space between words".into()),
                ));
                break;
            }
        }

        if results.is_empty() {
            // Try adding "man " prefix
            results.push(CorrectedCommand::new(
                format!("man {}", script),
                self.name(),
                1000,
                Some("Add man prefix".into()),
            ));
        }

        results
    }

    fn requires_output(&self) -> bool {
        true
    }
}
