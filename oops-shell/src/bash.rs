use crate::shell_trait::{Shell, ShellConfiguration};

pub struct Bash;

impl Shell for Bash {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn app_alias(&self, alias_name: &str) -> String {
        let bin = crate::resolve_bin_path();
        format!(
            "function {name}() {{\n\
    export TF_SHELL=bash;\n\
    export TF_ALIAS={name};\n\
    export TF_SHELL_ALIASES=$(alias);\n\
    export TF_HISTORY=$(fc -ln -10);\n\
    TF_CMD=$(\"{bin}\" --fix \"$@\") && eval \"$TF_CMD\";\n\
    unset TF_HISTORY;\n\
}}\n",
            name = alias_name,
            bin = bin,
        )
    }

    fn instant_mode_alias(&self, alias_name: &str) -> String {
        let bin = crate::resolve_bin_path();

        // When OOPS_INSTANT_MODE is already set, we're inside the PTY shell logger.
        // Modify PS1 to inject an invisible command-boundary marker so the log
        // reader can find individual command outputs.
        if std::env::var(oops_core::consts::ENV_INSTANT_MODE).is_ok() {
            // OSC 777 is ignored by all terminals — invisible marker in PS1.
            let mark = "\x1b]777;oops\x07";
            return format!(
                "export PS1=\"{mark}$PS1\";\n{}",
                self.app_alias(alias_name)
            );
        }

        // First invocation: start the shell logger, which replaces this shell
        // with a PTY session.  Inside the PTY, .bashrc is sourced again, and
        // this same alias will re-run — this time taking the branch above.
        format!(
            "export OOPS_INSTANT_MODE=true;\n\
export OOPS_OUTPUT_LOG=/tmp/oops-shell-log-$$;\n\
\"{bin}\" --shell-logger \"$OOPS_OUTPUT_LOG\"\n\
rm -f \"$OOPS_OUTPUT_LOG\"\n\
exit\n",
            bin = bin,
        )
    }

    fn split_command(&self, script: &str) -> Vec<String> {
        oops_core::command::split_script(script)
    }

    fn quote(&self, s: &str) -> String {
        if s.contains('\'') {
            format!("\"{}\"", s.replace('"', "\\\""))
        } else if s.contains(' ') || s.contains('$') || s.is_empty() {
            format!("'{}'", s)
        } else {
            s.to_string()
        }
    }

    fn history_command(&self) -> &str {
        "fc -ln -10"
    }

    fn config_file(&self) -> &str {
        resolve_bash_config()
    }

    fn history_put(&self, script: &str) -> String {
        format!("history -s {}\n", self.quote(script))
    }

    fn how_to_configure(&self) -> ShellConfiguration {
        let config = resolve_bash_config();
        ShellConfiguration {
            content: "eval \"$(command oops --alias)\"".to_string(),
            path: config.to_string(),
            reload: format!("source {}", config),
        }
    }
}

fn resolve_bash_config() -> &'static str {
    let home = dirs::home_dir().unwrap_or_default();
    if home.join(".bashrc").exists() {
        "~/.bashrc"
    } else if home.join(".bash_profile").exists() {
        "~/.bash_profile"
    } else {
        "~/.bashrc"
    }
}
