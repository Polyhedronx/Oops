// Rule modules
pub mod apt_get;
pub mod apt_get_search;
pub mod brew_install;
pub mod brew_unknown_command;
pub mod cargo_no_command;
pub mod cd_mkdir;
pub mod cd_parent;
pub mod chmod_x;
pub mod composer_command;
pub mod cp_omitting_dir;
pub mod docker_not_command;
pub mod git_add;
pub mod git_branch_delete;
pub mod git_branch_exists;
pub mod git_commit_amend;
pub mod git_merge;
pub mod git_not_command;
pub mod git_pull_uncommitted;
pub mod git_push_pull;
pub mod git_rm_staged;
pub mod git_stash;
pub mod grep_recursive;
pub mod ls_all;
pub mod man_no_space;
pub mod mkdir_p;
pub mod no_command;
pub mod no_such_file;
pub mod npm_run_script;
pub mod npm_wrong_command;
pub mod pip_install;
pub mod pip_unknown_command;
pub mod port_already_use;
pub mod python_execute;
pub mod rm_dir;
pub mod sl_ls;
pub mod ssh_known_hosts;
pub mod sudo;
pub mod systemctl;
pub mod touch;
pub mod unknown_command;

use oops_core::rule::Rule;
use once_cell::sync::Lazy;

/// The global rule registry. All built-in rules are registered here.
static REGISTRY: Lazy<Vec<Box<dyn Rule>>> = Lazy::new(|| {
    vec![
        // System rules (ordered by priority: low = runs first)
        Box::new(sudo::Sudo),
        Box::new(mkdir_p::MkdirP),
        Box::new(cd_mkdir::CdMkdir),
        Box::new(cd_parent::CdParent),
        Box::new(no_command::NoCommand),
        Box::new(unknown_command::UnknownCommand),
        Box::new(ssh_known_hosts::SshKnownHosts),
        Box::new(systemctl::Systemctl),
        Box::new(sl_ls::SlLs),
        Box::new(ls_all::LsAll),
        Box::new(python_execute::PythonExecute),
        Box::new(touch::Touch),
        Box::new(chmod_x::ChmodX),
        Box::new(rm_dir::RmDir),
        Box::new(cp_omitting_dir::CpOmittingDir),
        Box::new(man_no_space::ManNoSpace),
        Box::new(grep_recursive::GrepRecursive),
        Box::new(no_such_file::NoSuchFile),
        Box::new(port_already_use::PortAlreadyUse),
        // Package manager rules
        Box::new(apt_get::AptGet),
        Box::new(apt_get_search::AptGetSearch),
        Box::new(brew_install::BrewInstall),
        Box::new(brew_unknown_command::BrewUnknownCommand),
        Box::new(cargo_no_command::CargoNoCommand),
        Box::new(pip_install::PipInstall),
        Box::new(pip_unknown_command::PipUnknownCommand),
        Box::new(npm_run_script::NpmRunScript),
        Box::new(npm_wrong_command::NpmWrongCommand),
        Box::new(docker_not_command::DockerNotCommand),
        Box::new(composer_command::ComposerCommand),
        // Git rules
        Box::new(git_add::GitAdd),
        Box::new(git_push_pull::GitPushPull),
        Box::new(git_pull_uncommitted::GitPullUncommitted),
        Box::new(git_commit_amend::GitCommitAmend),
        Box::new(git_merge::GitMerge),
        Box::new(git_stash::GitStash),
        Box::new(git_rm_staged::GitRmStaged),
        Box::new(git_branch_delete::GitBranchDelete),
        Box::new(git_branch_exists::GitBranchExists),
        Box::new(git_not_command::GitNotCommand),
    ]
});

/// Get all registered rules.
pub fn get_all_rules() -> &'static [Box<dyn Rule>] {
    &REGISTRY
}
