// SPDX-FileCopyrightText: 2020 Ilaï Deutel & Kibi Contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

use crate::config::{self, parse_value as pv, parse_values as pvs};

/// Type of syntax highlighting for a single rendered character.
///
/// Each `HLType` is associated with a color, via its discriminant. The ANSI
/// color is equal to the discriminant, modulo 100. The colors are described
/// here: <https://en.wikipedia.org/wiki/ANSI_escape_code#Colors>
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    /// Write the ANSI color escape sequence for the `HLType` using the given
    /// formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { write!(f, "\x1b[{}m", (*self as u32) % 100) }
}

/// Configuration for syntax highlighting.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
    /// The token that start and end a multi-line strings, e.g. "\"\"\"" for
    /// Python.
    pub ml_string_delim: Option<String>,
    /// Keywords to highlight and their corresponding `HLType` (typically
    /// `HLType::Keyword1` or `HLType::Keyword2`)
    pub keywords: Vec<(HlType, Vec<String>)>,
}

impl Conf {
    /// Return the syntax configuration corresponding to the given file
    /// name, if a matching INI file is found in a config directory.
    /// If no matching configuration is found, return the default.
    pub fn find(name: &str, data_dirs: &[String]) -> Self {
        for data_dir in data_dirs {
            match PathBuf::from(data_dir).join("syntax.d").read_dir() {
                Ok(dir_entries) =>
                    for dir_entry in dir_entries {
                        match dir_entry.map(|dir_entry| Self::parse(&dir_entry.path())) {
                            // sfix = suffixes
                            Ok((sc, sfix)) if sfix.iter().any(|s| name.ends_with(s)) => return sc,
                            Ok((..)) => (),
                            Err(e) => eprintln!("Error iterating through {data_dir}/syntax.d: {e}"),
                        }
                    },
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => eprintln!("Error iterating through {data_dir}/syntax.d: {e}"),
            }
        }
        Self::default()
    }

    /// Load and parse a `SyntaxConf` from file.
    pub fn parse(path: &Path) -> (Self, Vec<String>) {
        let (mut sc, mut suffixes) = (Self::default(), Vec::new());
        config::process_ini_file(path, &mut |key, val| {
            match key {
                "name" => sc.name = pv(val)?,
                "extensions" => suffixes.extend(val.split(',').map(|u| format!(".{}", u.trim()))),
                "highlight_numbers" => sc.highlight_numbers = pv(val)?,
                "singleline_string_quotes" => sc.sl_string_quotes = pvs(val)?,
                "singleline_comment_start" => sc.sl_comment_start = pvs(val)?,
                "multiline_comment_delims" =>
                    sc.ml_comment_delims = match val.split_once(',') {
                        Some((v1, v2)) if !v2.contains(',') =>
                            Some((pv(v1.trim())?, pv(v2.trim())?)),
                        _ => return Err(format!("Expected 2 delimiters, got {val}")),
                    },
                "multiline_string_delim" => sc.ml_string_delim = Some(pv(val)?),
                "keywords_1" => sc.keywords.push((HlType::Keyword1, pvs(val)?)),
                "keywords_2" => sc.keywords.push((HlType::Keyword2, pvs(val)?)),
                _ => return Err(String::from("Invalid key")),
            }
            Ok(())
        });
        (sc, suffixes)
    }
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))] // No filesystem on wasm
mod tests {
    use std::collections::HashSet;
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    #[expect(clippy::assert_is_empty, reason = "Unnecessary for non-emptiness check.")]
    fn syntax_d_files() {
        let mut file_count = 0;
        let mut syntax_names = HashSet::new();
        for path in fs::read_dir("./syntax.d").unwrap() {
            let (conf, extensions) = Conf::parse(&path.unwrap().path());
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
        assert_eq!(Conf::parse(&tmp_path), (Conf::default(), Vec::<String>::new()));
    }

    #[test]
    fn full_example() {
        let tmp_dir = TempDir::new().expect("Could not create temporary directory");
        let file_path = tmp_dir.path().join("test_config.ini");
        let ini_content = r#"
name=Rust
extensions=rs
highlight_numbers=true
singleline_string_quotes= "
   singleline_comment_start=   //
multiline_comment_delims=/*,   */
; In Rust, the multi-line string delimiter is the same as the single-line string delimiter
multiline_string_delim="
; https://doc.rust-lang.org/book/appendix-01-keywords.html
keywords_1=abstract, as, async
keywords_2=i8, i16
"#;
        fs::write(&file_path, ini_content).expect("Could not write INI file");
        assert_eq!(
            Conf::parse(&file_path),
            (
                Conf {
                    name: String::from("Rust"),
                    highlight_numbers: true,
                    sl_string_quotes: vec!['"'],
                    sl_comment_start: vec![String::from("//")],
                    ml_comment_delims: Some((String::from("/*"), String::from("*/"))),
                    ml_string_delim: Some(String::from("\"")),
                    keywords: vec![
                        (HlType::Keyword1, vec![
                            String::from("abstract"),
                            String::from("as"),
                            String::from("async"),
                        ]),
                        (HlType::Keyword2, vec![String::from("i8"), String::from("i16"),])
                    ],
                },
                vec![String::from(".rs")]
            )
        );
    }
}
