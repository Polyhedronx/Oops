/// Default command wait timeout (seconds).
pub const DEFAULT_WAIT_COMMAND: u64 = 3;

/// Default slow command wait timeout (seconds).
pub const DEFAULT_WAIT_SLOW_COMMAND: u64 = 15;

/// Default config file name.
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Default history limit.
pub const DEFAULT_HISTORY_LIMIT: usize = 100;

/// Shell / instant-mode environment variables.
pub const ENV_SHELL: &str = "OOPS_SHELL";
pub const ENV_ALIAS: &str = "OOPS_ALIAS";
pub const ENV_HISTORY: &str = "OOPS_HISTORY";
pub const ENV_OUTPUT_LOG: &str = "OOPS_OUTPUT_LOG";
pub const ENV_INSTANT_MODE: &str = "OOPS_INSTANT_MODE";
