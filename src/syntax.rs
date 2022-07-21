use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

use crate::config::{self, parse_value as pv, parse_values as pvs};
use crate::{sys, Error};

/// Type of syntax highlighting for a single rendered character.
///
/// Each `HLType` is associated with a color, via its discriminant. The ANSI color is equal
/// to the discriminant, modulo 100. The colors are described here:
/// <https://en.wikipedia.org/wiki/ANSI_escape_code#Colors>
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum HlType {
    Normal = 39,     // Default foreground color
    Number = 31,     // Red
    Match = 46,      // Cyan
    String = 32,     // Green
    MlString = 132,  // Green
    Comment = 34,    // Blue
    MlComment = 134, // Blue
    Keyword1 = 33,   // Yellow
    Keyword2 = 35,   // Magenta
}

impl Display for HlType {
    /// Write the ANSI color escape sequence for the `HLType` using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { write!(f, "\x1b[{}m", (*self as u32) % 100) }
}

/// Configuration for syntax highlighting.
#[derive(Clone, Default)]
pub struct Conf {
    /// The name of the language, e.g. "Rust".
    pub name: String,
    /// Whether to highlight numbers.
    pub highlight_numbers: bool,
    /// Quotes for single-line strings.
    pub sl_string_quotes: Vec<char>,
    /// The tokens that starts a single-line comment, e.g. "//".
    pub sl_comment_start: Vec<String>,
    /// The tokens that start and end a multi-line comment, e.g. ("/*", "*/").
    pub ml_comment_delims: Option<(String, String)>,
    /// The token that start and end a multi-line strings, e.g. "\"\"\"" for Python.
    pub ml_string_delim: Option<String>,
    /// Keywords to highlight and there corresponding HLType (typically
    /// HLType::Keyword1 or HLType::Keyword2)
    pub keywords: Vec<(HlType, Vec<String>)>,
}

impl Conf {
    /// Return the syntax configuration corresponding to the given file extension, if a matching
    /// INI file is found in a config directory.
    pub fn get(ext: &str) -> Result<Option<Self>, Error> {
        for conf_dir in sys::data_dirs() {
            match PathBuf::from(conf_dir).join("syntax.d").read_dir() {
                Ok(dir_entries) =>
                    for dir_entry in dir_entries {
                        let (sc, extensions) = Self::from_file(&dir_entry?.path())?;
                        if extensions.into_iter().any(|e| e == ext) {
                            return Ok(Some(sc));
                        };
                    },
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(None)
    }
    /// Load a `SyntaxConf` from file.
    pub fn from_file(path: &Path) -> Result<(Self, Vec<String>), Error> {
        let (mut sc, mut extensions) = (Self::default(), Vec::new());
        config::process_ini_file(path, &mut |key, val| {
            match key {
                "name" => sc.name = pv(val)?,
                "extensions" => extensions.extend(val.split(',').map(|u| String::from(u.trim()))),
                "highlight_numbers" => sc.highlight_numbers = pv(val)?,
                "singleline_string_quotes" => sc.sl_string_quotes = pvs(val)?,
                "singleline_comment_start" => sc.sl_comment_start = pvs(val)?,
                "multiline_comment_delims" =>
                    sc.ml_comment_delims = match &val.split(',').collect::<Vec<_>>()[..] {
                        [v1, v2] => Some((pv(v1)?, pv(v2)?)),
                        d => return Err(format!("Expected 2 delimiters, got {}", d.len())),
                    },
                "multiline_string_delim" => sc.ml_string_delim = Some(pv(val)?),
                "keywords_1" => sc.keywords.push((HlType::Keyword1, pvs(val)?)),
                "keywords_2" => sc.keywords.push((HlType::Keyword2, pvs(val)?)),
                _ => return Err(format!("Invalid key: {}", key)),
            }
            Ok(())
        })?;
        Ok((sc, extensions))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn syntax_d_files() {
        let mut file_count = 0;
        let mut syntax_names = HashSet::new();
        for path in fs::read_dir("./syntax.d").unwrap() {
            let (conf, extensions) = Conf::from_file(&path.unwrap().path()).unwrap();
            assert!(!extensions.is_empty());
            syntax_names.insert(conf.name);
            file_count += 1;
        }
        assert!(file_count > 0);
        assert_eq!(file_count, syntax_names.len());
    }

    #[test]
    fn conf_from_invalid_path() {
        let tmp_dir = TempDir::new().expect("Could not create temporary directory");
        let tmp_path = tmp_dir.path().join("path_does_not_exist.ini");
        match Conf::from_file(&tmp_path) {
            Ok(_) => panic!("Conf::from_file should return an error"),
            Err(Error::Config(path, 0, _)) if path == tmp_path => (),
            Err(e) => panic!("Unexpected error {:?}", e),
        }
    }
}
