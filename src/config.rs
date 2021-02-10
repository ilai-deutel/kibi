//! # Configuration
//!
//! Utilities to configure the text editor.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::{fmt::Display, fs::File, str::FromStr, time::Duration};

use crate::{sys::conf_dirs as cdirs, Error, Error::Config as ConfErr};

/// The global Kibi configuration.
#[derive(Debug, PartialEq)]
pub struct Config {
    /// The size of a tab. Must be > 0.
    pub tab_stop: usize,
    /// The number of confirmations needed before quitting, when changes have been made since the
    /// file was last changed.
    pub quit_times: usize,
    /// The duration for which messages are shown in the status bar.
    pub message_dur: Duration,
    /// Whether to display line numbers.
    pub show_line_num: bool,
}

impl Default for Config {
    /// Default configuration.
    fn default() -> Self {
        Self { tab_stop: 4, quit_times: 2, message_dur: Duration::new(3, 0), show_line_num: true }
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
    ///     - `%APPDATA%\Kibi`
    ///
    /// # Errors
    ///
    /// Will return `Err` if one of the configuration file cannot be parsed properly.
    pub fn load() -> Result<Self, Error> {
        let mut conf = Self::default();

        let paths: Vec<_> = cdirs().iter().map(|d| PathBuf::from(d).join("config.ini")).collect();

        for path in paths.iter().filter(|p| p.is_file()).rev() {
            process_ini_file(path, &mut |key, value| {
                match key {
                    "tab_stop" => match parse_value(value)? {
                        0 => return Err("tab_stop must be > 0".into()),
                        tab_stop => conf.tab_stop = tab_stop,
                    },
                    "quit_times" => conf.quit_times = parse_value(value)?,
                    "message_duration" =>
                        conf.message_dur = Duration::from_secs_f32(parse_value(value)?),
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
pub fn process_ini_file<F>(path: &Path, kv_fn: &mut F) -> Result<(), Error>
where F: FnMut(&str, &str) -> Result<(), String> {
    let file = File::open(path).map_err(|e| ConfErr(path.into(), 0, e.to_string()))?;
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let (i, line) = (i + 1, line?);
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
pub fn parse_value<T: FromStr<Err = E>, E: Display>(value: &str) -> Result<T, String> {
    value.trim().parse().map_err(|e| format!("Parser error: {}", e))
}

/// Split a comma-separated list of values (right-hand side of a key=value1,value2,... INI line) and
/// parse it as a Vec.
pub fn parse_values<T: FromStr<Err = E>, E: Display>(value: &str) -> Result<Vec<T>, String> {
    value.split(',').map(parse_value).collect()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::{env, fs};

    use serial_test::serial;
    use tempfile::TempDir;

    use super::*;

    fn ini_processing_helper<F>(ini_content: &str, kv_fn: &mut F) -> Result<(), Error>
    where F: FnMut(&str, &str) -> Result<(), String> {
        let tmp_dir = TempDir::new().expect("Could not create temporary directory");
        let file_path = tmp_dir.path().join("test_config.ini");
        fs::write(&file_path, ini_content).expect("Could not write INI file");
        process_ini_file(&file_path, kv_fn)
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
            (String::from("a"), String::from(" c")),
            (String::from("variable"), String::from(" 4")),
            (String::from("a"), String::from(" d5")),
            (String::from("u"), String::from(" v = w ")),
        ];

        let mut kvs = Vec::new();
        let kv_fn = &mut |key: &str, value: &str| {
            kvs.push((String::from(key), String::from(value)));
            Ok(())
        };

        ini_processing_helper(ini_content, kv_fn).unwrap();

        assert_eq!(kvs, expected);
    }

    #[test]
    fn invalid_ini_processing() {
        let ini_content = "# Comment A
        ; Comment B
        a = c
            # Below is an empty line

           Invalid line
        a = d5
        u = v = w ";
        let kv_fn = &mut |_: &str, _: &str| Ok(());
        match ini_processing_helper(ini_content, kv_fn) {
            Ok(_) => panic!("process_ini_file should return an error"),
            Err(Error::Config(_, 6, s)) if s == "No '='" => (),
            Err(e) => panic!("Unexpected error {:?}", e),
        }
    }

    #[test]
    fn ini_processing_error_propagation() {
        let ini_content = "# Comment A
        ; Comment B
        a = c
            # Below is an empty line

           variable    = 4
        a = d5
        u = v = w ";
        let kv_fn = &mut |_: &str, _: &str| Err(String::from("test error"));
        match ini_processing_helper(ini_content, kv_fn) {
            Ok(_) => panic!("process_ini_file should return an error"),
            Err(Error::Config(_, 3, s)) if s == "test error" => (),
            Err(e) => panic!("Unexpected error {:?}", e),
        }
    }

    #[test]
    fn ini_processing_invalid_path() {
        let kv_fn = &mut |_: &str, _: &str| Ok(());
        let tmp_dir = TempDir::new().expect("Could not create temporary directory");
        let tmp_path = tmp_dir.path().join("path_does_not_exist.ini");
        match process_ini_file(&tmp_path, kv_fn) {
            Ok(_) => panic!("process_ini_file should return an error"),
            Err(Error::Config(path, 0, _)) if path == tmp_path => (),
            Err(e) => panic!("Unexpected error {:?}", e),
        }
    }

    fn test_config_dir(env_key: &OsStr, env_val: &OsStr, kibi_config_home: &Path) {
        let custom_config = Config { tab_stop: 99, quit_times: 50, ..Config::default() };
        let ini_content = format!(
            "# Configuration file
             tab_stop  = {}
             quit_times={}",
            custom_config.tab_stop, custom_config.quit_times
        );

        fs::create_dir_all(&kibi_config_home).unwrap();

        fs::write(kibi_config_home.join("config.ini"), ini_content)
            .expect("Could not write INI file");

        let config = Config::load().expect("Could not load configuration.");
        assert_ne!(config, custom_config);

        let config = {
            let orig_value = env::var_os(env_key);
            env::set_var(env_key, env_val);
            let config_res = Config::load();
            match orig_value {
                Some(orig_value) => env::set_var(env_key, orig_value),
                None => env::remove_var(env_key),
            }
            config_res.expect("Could not load configuration.")
        };

        assert_eq!(config, custom_config);
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn xdg_config_home() {
        let tmp_config_home = TempDir::new().expect("Could not create temporary directory");
        test_config_dir(
            "XDG_CONFIG_HOME".as_ref(),
            tmp_config_home.path().as_os_str(),
            &tmp_config_home.path().join("kibi"),
        );
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn config_home() {
        let tmp_home = TempDir::new().expect("Could not create temporary directory");
        test_config_dir(
            "HOME".as_ref(),
            tmp_home.path().as_os_str(),
            &tmp_home.path().join(".config/kibi"),
        );
    }

    #[cfg(windows)]
    #[test]
    #[serial]
    fn app_data() {
        let tmp_home = TempDir::new().expect("Could not create temporary directory");
        test_config_dir(
            "APPDATA".as_ref(),
            tmp_home.path().as_os_str(),
            &tmp_home.path().join("Kibi"),
        );
    }
}
