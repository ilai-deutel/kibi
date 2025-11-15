//! # Configuration
//!
//! Utilities to configure the text editor.

use std::path::{Path, PathBuf};
use std::{fmt::Display, fs::read_to_string, num::NonZeroUsize, str::FromStr, time::Duration};

use crate::sys::conf_dirs as cdirs;

/// The global Kibi configuration.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    /// The size of a tab. Must be > 0.
    pub tab_stop: NonZeroUsize,
    /// The number of confirmations needed before quitting, when changes have
    /// been made since the file was last changed.
    pub quit_times: usize,
    /// The duration for which messages are shown in the status bar.
    pub message_dur: Duration,
    /// Whether to display line numbers.
    pub show_line_num: bool,
}

impl Default for Config {
    /// Default configuration.
    fn default() -> Self {
        Self {
            #[expect(clippy::unwrap_used)]
            tab_stop: NonZeroUsize::new(4).unwrap(),
            quit_times: 2,
            message_dur: Duration::new(3, 0),
            show_line_num: true,
        }
    }
}

impl Config {
    /// Load the configuration, potentially overridden using `config.ini` files
    /// that can be located in the following directories:
    ///   - On Linux, macOS, and other *nix systems:
    ///     - `/etc/kibi` (system-wide configuration).
    ///     - `$XDG_CONFIG_HOME/kibi` if environment variable `$XDG_CONFIG_HOME`
    ///       is defined, `$HOME/.config/kibi` otherwise (user-level
    ///       configuration).
    ///   - On Windows:
    ///     - `%APPDATA%\Kibi`
    ///
    /// Will print warnings to stderr if a file or line cannot be parsed
    /// properly.
    pub fn load() -> Self {
        let mut conf = Self::default();

        let paths: Vec<_> = cdirs().iter().map(|d| PathBuf::from(d).join("config.ini")).collect();

        for path in paths.iter().filter(|p| p.is_file()).rev() {
            process_ini_file(path, &mut |key, value| {
                match key {
                    "tab_stop" => conf.tab_stop = parse_value(value)?,
                    "quit_times" => conf.quit_times = parse_value(value)?,
                    "message_duration" =>
                        conf.message_dur = Duration::try_from_secs_f32(parse_value(value)?)
                            .map_err(|x| x.to_string())?,
                    "show_line_numbers" => conf.show_line_num = parse_value(value)?,
                    _ => return Err(format!("Invalid key: {key}")),
                }
                Ok(())
            });
        }

        conf
    }
}

/// Process an INI file.
///
/// The `kv_fn` function will be called for each key-value pair in the file.
/// Typically, this function will update a configuration instance.
///
/// Will print warnings to stderr for invalid lines
pub fn process_ini_file<F>(path: &Path, kv_fn: &mut F)
where F: FnMut(&str, &str) -> Result<(), String> {
    read_to_string(path).map_or_else(
        |e| eprintln!("Could not read {}: {}", path.to_string_lossy(), e),
        |config| {
            for (i, line) in config.lines().enumerate().map(|(i, line)| (i, line.trim_start())) {
                let warn = |msg: &str| eprintln!("{}:{}: {}", path.to_string_lossy(), i + 1, msg);
                match (line.chars().next(), line.split_once('=')) {
                    (Some('#' | ';') | None, _) => (), // Comment or empty line
                    (_, Some((k, v))) => {
                        kv_fn(k.trim_end(), v.trim()).unwrap_or_else(|r| warn(&format!("{k}: {r}")))
                    }
                    (_, None) => warn("missing '='"),
                }
            }
        },
    );
}

/// Trim a value (right-hand side of a key=value INI line) and parses it.
pub fn parse_value<T: FromStr<Err=E>, E: Display>(value: &str) -> Result<T, String> {
    value.parse().map_err(|e: E| e.to_string())
}

/// Split a comma-separated list of values (right-hand side of a
/// key=value1,value2,... INI line) and parse it as a Vec.
pub fn parse_values<T: FromStr<Err=E>, E: Display>(values: &str) -> Result<Vec<T>, String> {
    values.split(',').map(|value| parse_value(value.trim())).collect()
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))] // No filesystem on wasm
mod tests {
    use std::collections::HashMap;
    use std::ffi::{OsStr, OsString};
    use std::sync::{LazyLock, Mutex, MutexGuard};
    use std::{env, fs};

    use tempfile::TempDir;

    use super::*;

    fn ini_processing_helper<F>(ini_content: &str, kv_fn: &mut F)
    where F: FnMut(&str, &str) -> Result<(), String> {
        let tmp_dir = TempDir::new().expect("Could not create temporary directory");
        let file_path = tmp_dir.path().join("test_config.ini");
        fs::write(&file_path, ini_content).expect("Could not write INI file");
        process_ini_file(&file_path, kv_fn);
    }

    #[test]
    fn valid_ini_processing() {
        let ini_content = "# Comment A
        ; Comment B
        a = c
            # Below is an empty line

           variable    = 4
        a = d5
        u = v = w ";
        let expected = vec![
            (String::from("a"), String::from("c")),
            (String::from("variable"), String::from("4")),
            (String::from("a"), String::from("d5")),
            (String::from("u"), String::from("v = w")),
        ];

        let mut kvs = Vec::new();
        let kv_fn = &mut |key: &str, value: &str| {
            kvs.push((String::from(key), String::from(value)));
            Ok(())
        };

        ini_processing_helper(ini_content, kv_fn);

        assert_eq!(kvs, expected);
    }

    #[test]
    fn ini_processing_with_invalid_line() {
        let ini_content = "# Comment A
        ; Comment B
        a = c
            # Below is an empty line

           Invalid line
        a = d5
        u = v = w ";
        let mut parsed: Vec<(String, String)> = vec![];
        let kv_fn = &mut |key: &str, value: &str| {
            parsed.push((key.into(), value.into()));
            Ok(())
        };
        ini_processing_helper(ini_content, kv_fn);
        assert_eq!(parsed, vec![
            (String::from("a"), String::from("c")),
            (String::from("a"), String::from("d5")),
            (String::from("u"), String::from("v = w"))
        ]);
    }
    #[test]
    fn ini_processing_invalid_path() {
        let kv_fn = &mut |_: &str, _: &str| panic!("Should not be called");
        let tmp_dir = TempDir::new().expect("Could not create temporary directory");
        let tmp_path = tmp_dir.path().join("path_does_not_exist.ini");
        process_ini_file(&tmp_path, kv_fn);
    }

    /// Lock for modifying environment variables.
    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

    struct TempEnvVars<'a> {
        original_values: HashMap<&'static OsStr, Option<OsString>>,
        _lock: MutexGuard<'a, ()>,
    }

    impl TempEnvVars<'_> {
        fn new() -> Self {
            Self {
                original_values: HashMap::new(),
                _lock: ENV_LOCK.lock().expect("Could not acquire lock."),
            }
        }

        fn set(&mut self, key: &'static OsStr, value: Option<&OsStr>) {
            let original_value = env::var_os(key);
            assert!(self.original_values.insert(key, original_value).is_none());
            // SAFETY: Only one test at a time may set or remove an environment variable, as
            // enforced by ENV_LOCK.
            #[expect(unsafe_code)]
            unsafe {
                match value {
                    Some(value) => env::set_var(key, value),
                    None => env::remove_var(key),
                }
            }
        }
    }

    impl Drop for TempEnvVars<'_> {
        fn drop(&mut self) {
            // SAFETY: Only one test at a time may set or remove an environment variable, as
            // enforced by ENV_LOCK.
            #[expect(unsafe_code)]
            unsafe {
                for (key, original_value) in &self.original_values {
                    match original_value {
                        Some(original_value) => env::set_var(key, original_value),
                        None => env::remove_var(key),
                    }
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    #[expect(clippy::significant_drop_tightening, reason = "False positive")]
    fn invalid_tab_stop() {
        let tmp_config_home = TempDir::new().expect("Could not create temporary directory");

        let mut vars = TempEnvVars::new();
        vars.set(OsStr::new("XDG_CONFIG_HOME"), Some(tmp_config_home.path().as_os_str()));

        let kibi_config_home = tmp_config_home.path().join("kibi");
        fs::create_dir_all(&kibi_config_home).unwrap();
        fs::write(kibi_config_home.join("config.ini"), "tab_stop=0")
            .expect("Could not write INI file");

        let config = Config::load();
        // Tab stop value is still the default
        assert_eq!(config.tab_stop.get(), 4);
    }

    fn test_config_dir(
        env_key: &'static OsStr, env_val: &OsStr, kibi_config_home: &Path, vars: &mut TempEnvVars,
    ) {
        let custom_config = Config {
            tab_stop: NonZeroUsize::new(99).unwrap(),
            quit_times: 50,
            ..Config::default()
        };
        let ini_content = format!(
            "# Configuration file
             tab_stop  = {}
             quit_times={}",
            custom_config.tab_stop, custom_config.quit_times
        );

        fs::create_dir_all(kibi_config_home).unwrap();

        fs::write(kibi_config_home.join("config.ini"), ini_content)
            .expect("Could not write INI file");

        let config = Config::load();
        assert_ne!(config, custom_config);

        vars.set(env_key, Some(env_val));
        let config = Config::load();

        assert_eq!(config, custom_config);
    }

    #[cfg(unix)]
    #[test]
    fn xdg_config_home() {
        let mut vars = TempEnvVars::new();
        let tmp_config_home = TempDir::new().expect("Could not create temporary directory");
        test_config_dir(
            OsStr::new("XDG_CONFIG_HOME"),
            tmp_config_home.path().as_os_str(),
            &tmp_config_home.path().join("kibi"),
            &mut vars,
        );
    }

    #[expect(clippy::significant_drop_tightening, reason = "Lock is needed until the end")]
    #[cfg(unix)]
    #[test]
    fn config_home() {
        let mut vars = TempEnvVars::new();
        vars.set(OsStr::new("XDG_CONFIG_HOME"), None);
        let tmp_home = TempDir::new().expect("Could not create temporary directory");
        test_config_dir(
            OsStr::new("HOME"),
            tmp_home.path().as_os_str(),
            &tmp_home.path().join(".config/kibi"),
            &mut vars,
        );
    }

    #[cfg(windows)]
    #[test]
    fn app_data() {
        let mut vars = TempEnvVars::new();
        let tmp_home = TempDir::new().expect("Could not create temporary directory");
        test_config_dir(
            OsStr::new("APPDATA"),
            tmp_home.path().as_os_str(),
            &tmp_home.path().join("Kibi"),
            &mut vars,
        );
    }
}
