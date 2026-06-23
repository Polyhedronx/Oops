use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::consts;

/// Main configuration for oops.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Rules configuration: either "all" or a list of rule names/globs.
    #[serde(default = "default_rules_config")]
    pub rules: RulesConfig,

    /// Rules to always exclude.
    #[serde(default)]
    pub exclude_rules: Vec<String>,

    /// Priority overrides: rule_name -> priority (lower = shown first).
    #[serde(default)]
    pub priority_overrides: HashMap<String, i32>,

    /// Seconds to wait for command output capture.
    #[serde(default = "default_wait_command")]
    pub wait_command: u64,

    /// Seconds to wait for slow commands.
    #[serde(default = "default_wait_slow_command")]
    pub wait_slow_command: u64,

    /// Commands considered "slow" (longer timeout applies).
    #[serde(default = "default_slow_commands")]
    pub slow_commands: Vec<String>,

    /// Require user confirmation via TUI (false = auto-select first).
    #[serde(default = "default_true")]
    pub require_confirmation: bool,

    /// Wrap corrected command to retry oops on failure.
    #[serde(default)]
    pub repeat: bool,

    /// Add corrected command to shell history.
    #[serde(default = "default_true")]
    pub alter_history: bool,

    /// Enable debug logging.
    #[serde(default)]
    pub debug: bool,

    /// Maximum history entries to consider.
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,

    /// Environment variables to set when re-running commands.
    #[serde(default)]
    pub env: HashMap<String, String>,
}

fn default_rules_config() -> RulesConfig {
    RulesConfig::All
}

fn default_wait_command() -> u64 {
    consts::DEFAULT_WAIT_COMMAND
}

fn default_wait_slow_command() -> u64 {
    consts::DEFAULT_WAIT_SLOW_COMMAND
}

fn default_slow_commands() -> Vec<String> {
    vec![
        "lein".into(),
        "react-native".into(),
        "gradle".into(),
        "./gradlew".into(),
        "vagrant".into(),
    ]
}

fn default_true() -> bool {
    true
}

fn default_history_limit() -> usize {
    consts::DEFAULT_HISTORY_LIMIT
}

/// Rules configuration: either all enabled, or a specific list.
#[derive(Debug, Clone, Default)]
pub enum RulesConfig {
    #[default]
    All,
    List(Vec<String>),
}

impl Serialize for RulesConfig {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            RulesConfig::All => serializer.serialize_str("all"),
            RulesConfig::List(list) => list.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for RulesConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RulesConfigVisitor;
        impl<'de> serde::de::Visitor<'de> for RulesConfigVisitor {
            type Value = RulesConfig;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(r#""all" or a list of rule names"#)
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                if v.eq_ignore_ascii_case("all") || v == "*" {
                    Ok(RulesConfig::All)
                } else {
                    Ok(RulesConfig::List(vec![v.to_string()]))
                }
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
                let list = Vec::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))?;
                Ok(RulesConfig::List(list))
            }
        }
        deserializer.deserialize_any(RulesConfigVisitor)
    }
}

impl Config {
    /// Load configuration from the default path, or return defaults.
    pub fn load() -> Self {
        let config_path = Self::default_path();
        Self::load_from(config_path.as_deref())
    }

    /// Load configuration from an optional path. Falls back to defaults.
    pub fn load_from(path: Option<&std::path::Path>) -> Self {
        let mut config: Config = path
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();

        // Apply environment variable overrides
        config.apply_env_overrides();
        config
    }

    /// Apply OOPS_* environment variable overrides.
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("OOPS_REQUIRE_CONFIRMATION") {
            if let Ok(b) = val.parse() {
                self.require_confirmation = b;
            }
        }
        if let Ok(val) = std::env::var("OOPS_DEBUG") {
            if let Ok(b) = val.parse() {
                self.debug = b;
            }
        }
        if let Ok(val) = std::env::var("OOPS_REPEAT") {
            if let Ok(b) = val.parse() {
                self.repeat = b;
            }
        }
        if let Ok(val) = std::env::var("OOPS_ALTER_HISTORY") {
            if let Ok(b) = val.parse() {
                self.alter_history = b;
            }
        }
        if let Ok(val) = std::env::var("OOPS_WAIT_COMMAND") {
            if let Ok(n) = val.parse() {
                self.wait_command = n;
            }
        }
        if let Ok(val) = std::env::var("OOPS_RULES") {
            let list: Vec<String> = val.split(',').map(|s| s.trim().to_string()).collect();
            if !list.is_empty() {
                self.rules = RulesConfig::List(list);
            }
        }
    }

    /// Get the default configuration file path.
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("oops").join(consts::CONFIG_FILE_NAME))
    }

    /// Check if a rule with the given name is enabled.
    pub fn is_rule_enabled(&self, name: &str, enabled_by_default: bool) -> bool {
        // If explicitly excluded, never enabled
        if self.exclude_rules.iter().any(|e| glob_match(e, name)) {
            return false;
        }

        match &self.rules {
            RulesConfig::All => enabled_by_default,
            RulesConfig::List(list) => list.iter().any(|e| glob_match(e, name)),
        }
    }

    /// Get the priority for a rule, applying overrides.
    pub fn get_priority(&self, name: &str, default_priority: i32) -> i32 {
        self.priority_overrides
            .get(name)
            .copied()
            .unwrap_or(default_priority)
    }

    /// Check if a command is in the slow commands list.
    pub fn is_slow_command(&self, script: &str) -> bool {
        self.slow_commands
            .iter()
            .any(|slow| script.starts_with(slow))
    }

    /// Create a default config (used by `oops init`).
    pub fn default_config() -> Self {
        Self::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rules: RulesConfig::All,
            exclude_rules: Vec::new(),
            priority_overrides: HashMap::new(),
            wait_command: consts::DEFAULT_WAIT_COMMAND,
            wait_slow_command: consts::DEFAULT_WAIT_SLOW_COMMAND,
            slow_commands: default_slow_commands(),
            require_confirmation: true,
            repeat: false,
            alter_history: true,
            debug: false,
            history_limit: consts::DEFAULT_HISTORY_LIMIT,
            env: HashMap::new(),
        }
    }
}

/// Simple glob matching for rule names.
/// Supports `*` wildcard and literal matching.
fn glob_match(pattern: &str, name: &str) -> bool {
    if pattern == "*" || pattern == "all" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == name;
    }
    let prefix = pattern.trim_end_matches('*');
    name.starts_with(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_exact_match() {
        assert!(glob_match("git_not_command", "git_not_command"));
        assert!(!glob_match("git_not_command", "git_push"));
    }

    #[test]
    fn test_glob_wildcard() {
        assert!(glob_match("git_*", "git_not_command"));
        assert!(glob_match("git_*", "git_push"));
        assert!(!glob_match("git_*", "sudo"));
    }

    #[test]
    fn test_glob_all() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("all", "anything"));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(matches!(config.rules, RulesConfig::All));
        assert!(config.require_confirmation);
    }
}
