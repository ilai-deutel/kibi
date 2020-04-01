//! # Configuration
//!
//! Utilities to configure the text editor.

use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use crate::{sys::conf_dirs, Error, Error::Config as ConfErr};

/// The global Kibi configuration.
pub struct Config {
    /// The size of a tab. Must be > 0.
    pub(crate) tab_stop: usize,
    /// The number of confirmations needed before quitting, when changes have been made since the
    /// file was last changed.
    pub(crate) quit_times: usize,
    /// The duration for which messages are shown in the status bar.
    pub(crate) message_duration: Duration,
    /// Whether to display line numbers.
    pub(crate) show_line_num: bool,
    /// The paths to directories that may be used to store Kibi configuration files.
    pub(crate) conf_dirs: Vec<PathBuf>,
}

/// Return directories that may contain configuration files for Kibi
fn kibi_conf_dirs() -> Vec<PathBuf> {
    conf_dirs().iter().filter_map(Option::as_ref).map(|p| PathBuf::from(p).join("kibi")).collect()
}

impl Default for Config {
    /// Default configuration.
    fn default() -> Self {
        Self {
            tab_stop: 4,
            quit_times: 2,
            message_duration: Duration::from_secs(3),
            show_line_num: true,
            conf_dirs: kibi_conf_dirs(),
        }
    }
}

impl Config {
    /// Load the configuration, potentially overridden using `config.ini` files that can be located
    /// in the following directories:
    ///   - On Linux, macOS, and other *nix systems:
    ///     - `/etc/kibi` (system-wide configuration).
    ///     - `$XDG_CONFIG_HOME/kibi` if environment variable `$XDG_CONFIG_HOME` is defined,
    ///       `$HOME/.config/kibi` otherwise (user-level configuration).
    ///   - On Windows:
    ///     - `%APPDATA%/kibi`
    ///
    /// # Errors
    ///
    /// Will return `Err` if one of the configuration file cannot be parsed properly.
    pub fn load() -> Result<Self, Error> {
        let mut conf = Self::default();

        let conf_paths: Vec<_> = conf.conf_dirs.iter().map(|p| p.join("config.ini")).collect();

        for path in conf_paths.into_iter().filter(|p| p.exists()) {
            process_ini_file(&path, &mut |key, value| {
                match key {
                    "tab_stop" => match parse_value(value)? {
                        0 => return Err("tab_stop must be > 0".into()),
                        tab_stop => conf.tab_stop = tab_stop,
                    },
                    "quit_times" => conf.quit_times = parse_value(value)?,
                    "message_duration" =>
                        conf.message_duration = Duration::from_secs_f32(parse_value(value)?),
                    "show_line_numbers" => conf.show_line_num = parse_value(value)?,
                    _ => return Err(format!("Invalid key: {}", key)),
                };
                Ok(())
            })?;
        }

        Ok(conf)
    }
}

/// Process an INI file.
///
/// The `kv_fn` function will be called for each key-value pair in the file. Typically, this
/// function will update a configuration instance.
pub(crate) fn process_ini_file<F>(path: &Path, kv_fn: &mut F) -> Result<(), Error>
where F: FnMut(&str, &str) -> Result<(), String> {
    for (i, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        let mut parts = line.trim_start().splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some(comment_line), _) if comment_line.starts_with(&['#', ';'][..]) => (),
            (Some(k), Some(v)) => kv_fn(k.trim_end(), v).map_err(|r| ConfErr(path.into(), i, r))?,
            (Some(""), None) | (None, _) => (), // Empty line
            (Some(_), None) => return Err(ConfErr(path.into(), i, String::from("No '='"))),
        }
    }
    Ok(())
}

/// Trim a value (right-hand side of a key=value INI line) and parses it.
pub(crate) fn parse_value<T: FromStr<Err = E>, E: Display>(value: &str) -> Result<T, String> {
    value.trim().parse().map_err(|e| format!("Parser error: {}", e))
}

/// Split a comma-separated list of values (right-hand side of a key=value1,value2,... INI line) and
/// parse it as a Vec.
pub(crate) fn parse_values<T: FromStr<Err = E>, E: Display>(value: &str) -> Result<Vec<T>, String> {
    value.split(',').map(parse_value).collect()
}
