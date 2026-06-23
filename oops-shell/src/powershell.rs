use crate::shell_trait::{Shell, ShellConfiguration};

pub struct PowerShell;

impl Shell for PowerShell {
    fn name(&self) -> &'static str {
        "powershell"
    }

    fn app_alias(&self, alias_name: &str) -> String {
        let abs_bin = crate::resolve_bin_path();
        format!(
            "function {name} {{\n\
    $env:TF_SHELL = \"powershell\"\n\
    $env:TF_ALIAS = \"{name}\"\n\
    $last = (Get-History -Count 1 -ErrorAction SilentlyContinue)\n\
    if ($last) {{\n\
        $env:OOPS_HISTORY = $last.CommandLine\n\
    }}\n\
    $bin = if (Test-Path \"{abs_bin}\") {{ \"{abs_bin}\" }} else {{ \"oops\" }}\n\
    $tmp = New-TemporaryFile\n\
    $env:OOPS_OUTPUT_FILE = $tmp.FullName\n\
    & $bin --fix @args\n\
    if (Test-Path $tmp.FullName) {{\n\
        $result = Get-Content $tmp.FullName -Raw\n\
        Remove-Item $tmp.FullName -ErrorAction SilentlyContinue\n\
        if ($result) {{ Invoke-Expression $result }}\n\
    }}\n\
    Remove-Item Env:\\OOPS_HISTORY -ErrorAction SilentlyContinue\n\
    Remove-Item Env:\\OOPS_OUTPUT_FILE -ErrorAction SilentlyContinue\n\
    [Console]::ResetColor()\n\
}}\n",
            name = alias_name,
            abs_bin = abs_bin,
        )
    }

    fn instant_mode_alias(&self, alias_name: &str) -> String {
        self.app_alias(alias_name)
    }

    fn split_command(&self, script: &str) -> Vec<String> {
        oops_core::command::split_script(script)
    }

    fn quote(&self, s: &str) -> String {
        if s.contains('"') {
            format!("'{}'", s)
        } else if s.contains(' ') || s.contains('$') || s.is_empty() {
            format!("\"{}\"", s)
        } else {
            s.to_string()
        }
    }

    fn history_command(&self) -> &str {
        "Get-History -Count 10 | ForEach-Object { $_.CommandLine }"
    }

    fn config_file(&self) -> &str {
        // Return the *literal* PowerShell variable reference.
        // The CLI resolves it via pwsh.exe before writing.
        "$PROFILE"
    }

    fn history_put(&self, script: &str) -> String {
        format!(
            "Add-Content -Path (Get-PSReadlineOption).HistorySavePath -Value {}\n",
            self.quote(script)
        )
    }

    fn how_to_configure(&self) -> ShellConfiguration {
        let bin = crate::resolve_bin_path();
        ShellConfiguration {
            content: format!(
                "& \"{}\" --alias | Out-String | Invoke-Expression",
                bin
            ),
            path: "$PROFILE".to_string(),
            reload: ". $PROFILE".to_string(),
        }
    }
}
