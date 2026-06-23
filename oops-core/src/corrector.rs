use std::collections::HashSet;

use tracing::debug;

use crate::command::Command;
use crate::config::Config;
use crate::corrected_command::CorrectedCommand;
use crate::rule::Rule;

/// The core engine that matches commands against rules and collects corrections.
pub struct Corrector<'a> {
    rules: &'a [Box<dyn Rule>],
    config: Config,
}

impl<'a> Corrector<'a> {
    /// Create a new Corrector with the given rules and config.
    pub fn new(rules: &'a [Box<dyn Rule>], config: Config) -> Self {
        Self { rules, config }
    }

    /// Find all matching corrections for the given command.
    /// Results are deduplicated by script and sorted by priority (lowest first).
    pub fn get_corrected_commands(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results: Vec<CorrectedCommand> = Vec::new();

        for rule in self.rules {
            // Skip if rule is not enabled
            if !self
                .config
                .is_rule_enabled(rule.name(), rule.enabled_by_default())
            {
                debug!("Rule '{}' is disabled, skipping", rule.name());
                continue;
            }

            // Skip if rule requires output but none is available
            if rule.requires_output() && command.output.is_none() {
                debug!(
                    "Rule '{}' requires output but none available, skipping",
                    rule.name()
                );
                continue;
            }

            // Check if rule matches
            if !rule.match_command(command) {
                continue;
            }

            debug!("Rule '{}' matched!", rule.name());

            // Get corrected commands from this rule
            let corrected = rule.get_new_command(command);
            for mut cmd in corrected {
                // Apply user priority overrides
                cmd.priority = self.config.get_priority(cmd.rule_name, cmd.priority);
                results.push(cmd);
            }
        }

        // Deduplicate by script, keeping the one with lowest priority
        let mut seen: HashSet<String> = HashSet::new();
        let mut unique: Vec<CorrectedCommand> = Vec::new();

        // Sort by priority first so lower-priority ones come first
        results.sort_by_key(|c| c.priority);

        for cmd in results {
            if seen.insert(cmd.script.clone()) {
                unique.push(cmd);
            }
        }

        debug!(
            "Found {} unique corrections from {} results",
            unique.len(),
            seen.len()
        );

        unique
    }

    /// Get a reference to the rules.
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        self.rules
    }

    /// Get a reference to the config.
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    struct AlwaysMatchRule {
        name: &'static str,
        script: String,
    }

    impl Rule for AlwaysMatchRule {
        fn name(&self) -> &'static str {
            self.name
        }

        fn match_command(&self, _command: &Command) -> bool {
            true
        }

        fn get_new_command(&self, _command: &Command) -> Vec<CorrectedCommand> {
            vec![CorrectedCommand {
                script: self.script.clone(),
                rule_name: self.name(),
                priority: 1000,
                description: Some("test".into()),
            }]
        }

        fn requires_output(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_corrector_dedup() {
        let rules: Vec<Box<dyn Rule>> = vec![
            Box::new(AlwaysMatchRule {
                name: "rule_a",
                script: "corrected".into(),
            }),
            Box::new(AlwaysMatchRule {
                name: "rule_b",
                script: "corrected".into(),
            }),
        ];
        let corrector = Corrector::new(&rules, Config::default());
        let cmd = Command::new("fail", None);
        let results = corrector.get_corrected_commands(&cmd);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].script, "corrected");
    }

    #[test]
    fn test_corrector_no_match() {
        struct NoMatchRule;
        impl Rule for NoMatchRule {
            fn name(&self) -> &'static str {
                "no_match"
            }
            fn match_command(&self, _command: &Command) -> bool {
                false
            }
            fn get_new_command(&self, _command: &Command) -> Vec<CorrectedCommand> {
                vec![]
            }
        }

        let rules: Vec<Box<dyn Rule>> = vec![Box::new(NoMatchRule)];
        let corrector = Corrector::new(&rules, Config::default());
        let cmd = Command::new("anything", None);
        let results = corrector.get_corrected_commands(&cmd);
        assert!(results.is_empty());
    }
}
