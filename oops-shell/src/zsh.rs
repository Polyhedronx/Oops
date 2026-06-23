use crate::shell_trait::{Shell, ShellConfiguration};

pub struct Zsh;

impl Shell for Zsh {
    fn name(&self) -> &'static str {
        "zsh"
    }

    fn app_alias(&self, alias_name: &str) -> String {
        let bin = crate::resolve_bin_path();
        format!(
            "function {name}() {{\n\
    export TF_SHELL=zsh;\n\
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

        // Already inside the PTY shell logger — modify PS1.
        if std::env::var(oops_core::consts::ENV_INSTANT_MODE).is_ok() {
            // Zsh needs %{...%} around escape sequences so it doesn't count them
            // in prompt width calculations.
            let mark = "%{\x1b]777;oops\x07%}";
            return format!(
                "export PS1=\"{mark}$PS1\";\n{}",
                self.app_alias(alias_name)
            );
        }

        // First invocation: start the shell logger PTY session.
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
        "~/.zshrc"
    }

    fn history_put(&self, script: &str) -> String {
        format!("print -s {}\n", self.quote(script))
    }

    fn how_to_configure(&self) -> ShellConfiguration {
        ShellConfiguration {
            content: "eval \"$(command oops --alias)\"".to_string(),
            path: "~/.zshrc".to_string(),
            reload: "source ~/.zshrc".to_string(),
        }
    }
}
