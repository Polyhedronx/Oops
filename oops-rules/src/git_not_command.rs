use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `git` commands when a subcommand is not found (suggest similar commands).
pub struct GitNotCommand;

/// Check if a git subcommand is a known common typo.
fn is_known_typo(cmd: &str) -> bool {
    let typos: &[&str] = &[
        "pish", "pul", "addd", "comit", "commmit",
        "chekcout", "checkot", "checout", "brnach", "brach",
        "merg", "statsu", "stastus", "stah", "stahs", "dff", "lg",
    ];
    typos.contains(&cmd)
}

impl Rule for GitNotCommand {
    fn name(&self) -> &'static str {
        "git_not_command"
    }

    fn match_command(&self, command: &Command) -> bool {
        let is_git = command
            .script_parts()
            .first()
            .is_some_and(|first| first == "git");

        if !is_git {
            return false;
        }

        // Match on known typos even without output
        let parts: Vec<&str> = command.script.split_whitespace().collect();
        if parts.len() >= 2 && is_known_typo(parts[1]) {
            return true;
        }

        // Also match on error output if available
        command.output.as_ref().is_some_and(|out| {
            let lower = out.to_lowercase();
            (lower.contains("is not a git command")
                || lower.contains("unknown command")
                || lower.contains("did you mean"))
                && !lower.contains("see 'git help")
        })
    }

    fn requires_output(&self) -> bool {
        false
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let parts: Vec<&str> = command.script.split_whitespace().collect();
        if parts.len() < 2 {
            return vec![];
        }

        // Common git subcommand typos
        let mut results = Vec::new();
        let fixes: &[(&str, &str)] = &[
            ("pish", "push"),
            ("pul", "pull"),
            ("addd", "add"),
            ("comit", "commit"),
            ("commmit", "commit"),
            ("chekcout", "checkout"),
            ("checkot", "checkout"),
            ("checout", "checkout"),
            ("brnach", "branch"),
            ("brach", "branch"),
            ("merg", "merge"),
            ("statsu", "status"),
            ("stastus", "status"),
            ("stah", "stash"),
            ("stahs", "stash"),
            ("diff ", "diff "),
            ("dff", "diff"),
            ("lg", "log"),
        ];

        let cmd = parts[1];
        for (typo, correct) in fixes {
            if cmd == *typo {
                let corrected = format!(
                    "git {} {}",
                    correct,
                    parts[2..].join(" ")
                );
                results.push(CorrectedCommand::new(
                    corrected.trim().to_string(),
                    self.name(),
                    1000,
                    Some(format!("Replace '{}' with '{}'", typo, correct)),
                ));
            }
        }

        // Also try suggesting the closest git command if nothing matched
        if results.is_empty() {
            use strsim::damerau_levenshtein;
            let known: &[&str] = &[
                "add", "am", "archive", "bisect", "blame", "branch",
                "checkout", "cherry-pick", "clean", "clone", "commit",
                "config", "describe", "diff", "fetch", "format-patch",
                "fsck", "gc", "grep", "init", "log", "merge", "mv",
                "notes", "pull", "push", "rebase", "reflog", "remote",
                "reset", "restore", "revert", "rm", "shortlog", "show",
                "stash", "status", "submodule", "switch", "tag",
            ];

            let mut best: Option<(&str, usize)> = None;
            for known_cmd in known {
                let dist = damerau_levenshtein(cmd, known_cmd);
                if dist <= 2 && dist < cmd.len().min(known_cmd.len()).max(1)
                    && best.is_none_or(|(_, d)| dist < d)
                {
                    best = Some((known_cmd, dist));
                }
            }

            if let Some((suggestion, _)) = best {
                if suggestion != cmd {
                    results.push(CorrectedCommand::new(
                        format!("git {} {}", suggestion, parts[2..].join(" ")).trim().to_string(),
                        self.name(),
                        1000,
                        Some(format!("Did you mean '{}'?", suggestion)),
                    ));
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_pish() {
        let cmd = Command::new(
            "git pish origin main",
            Some("git: 'pish' is not a git command. See 'git --help'.".into()),
        );
        assert!(GitNotCommand.match_command(&cmd));
        let results = GitNotCommand.get_new_command(&cmd);
        assert_eq!(results[0].script, "git push origin main");
    }

    #[test]
    fn test_git_suggestion() {
        let cmd = Command::new(
            "git chekcout main",
            Some("git: 'chekcout' is not a git command. See 'git --help'.".into()),
        );
        assert!(GitNotCommand.match_command(&cmd));
        let results = GitNotCommand.get_new_command(&cmd);
        // Should suggest checkout
        assert!(results.iter().any(|c| c.script.contains("checkout")));
    }
}
