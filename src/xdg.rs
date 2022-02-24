use std::env::var;

/// Return directories following the XDG Base Directory Specification
///
/// See `conf_dirs()` and `data_dirs()` for usage example.
///
/// The XDG Base Directory Specification is defined here:
/// <https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html>
pub(crate) fn xdg_dirs(xdg_type: &str, def_home_suffix: &str, def_dirs: &str) -> Vec<String> {
    let (home_key, dirs_key) = (format!("XDG_{}_HOME", xdg_type), format!("XDG_{}_DIRS", xdg_type));

    let mut dirs = Vec::new();

    // If environment variable `home_key` (e.g. `$XDG_CONFIG_HOME`) is set, add its value to `dirs`.
    // Otherwise, if environment variable `$HOME` is set, add `$HOME{def_home_suffix}`
    // (e.g. `$HOME/.config`) to `dirs`.
    dirs.extend(var(home_key).or_else(|_| var("HOME").map(|d| d + def_home_suffix)).into_iter());

    // If environment variable `dirs_key` (e.g. `XDG_CONFIG_DIRS`) is set, split by `:` and add the
    // parts to `dirs`.
    // Otherwise, add the split `def_dirs` (e.g. `/etc/xdg:/etc`) and add the parts to `dirs`.
    dirs.extend(var(dirs_key).unwrap_or_else(|_| def_dirs.into()).split(':').map(String::from));

    dirs.into_iter().map(|p| p + "/kibi").collect()
}

/// Return configuration directories for UNIX systems
pub fn conf_dirs() -> Vec<String> { xdg_dirs("CONFIG", "/.config", "/etc/xdg:/etc") }

/// Return syntax directories for UNIX systems
pub fn data_dirs() -> Vec<String> {
    xdg_dirs("DATA", "/.local/share", "/usr/local/share/:/usr/share/")
}