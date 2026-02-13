use std::{ffi::OsStr, path::Path};

use tokei::LanguageType;

use crate::{BOLD, GREEN, RED, RESET, Result};

const INNER_ATTRIBUTE_PREFIX: &str = "#![";
const OUTER_ATTRIBUTE_PREFIX: &str = "#[";
const ATTRIBUTE_PREFIXES_TO_IGNORE: [&str; 7] = [
    // Lint directives
    "allow(",
    "warn(",
    "deny(",
    "expect(",
    "#[cfg_attr(any(windows, target_os = \"wasi\"), expect(",
    // Test-only attributes
    "cfg_attr(test,",
    "cfg_attr(fuzzing,",
];

fn count_file_loc(path: &Path, config: &tokei::Config) -> Result<usize> {
    if path.extension() != Some(OsStr::new("rs")) {
        Err(format!("{path} is not a Rust file", path = path.display()))?;
    }

    let source = filter_lines(path)?;
    let stats = LanguageType::Rust.parse_from_str(source, config);

    Ok(stats.code)
}

pub fn count_loc() -> Result<()> {
    let config = tokei::Config::default();
    let source_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("Could not find source directory")?
        .join("src");

    let mut results = Vec::new();
    for entry in std::fs::read_dir(&source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            Err(format!("Subdirectories are not supported: {path}", path = path.display()))?;
        }

        let count = count_file_loc(&path, &config)?;
        let file_name = path.strip_prefix(&source_dir)?.display().to_string();
        results.push((file_name, count));
    }
    results.sort();

    print_summary(&results, &["unix", "wasi", "windows"])
}

/// Filter out lines that contain lints and anything after
/// `#[cfg(test)]` attributes.
fn filter_lines(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let lines = content
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            // TODO: Use strip_prefix_of when https://github.com/rust-lang/rfcs/pull/528 is stabilized
            line.strip_prefix(INNER_ATTRIBUTE_PREFIX)
                .or_else(|| line.strip_prefix(OUTER_ATTRIBUTE_PREFIX))
                .filter(|s| ATTRIBUTE_PREFIXES_TO_IGNORE.iter().any(|prefix| s.starts_with(prefix)))
                .is_none()
        })
        .take_while(|line| !line.contains("#[cfg(test)]"));
    let filtered_content = lines.collect::<Vec<_>>().join("\n");
    Ok(filtered_content)
}

fn print_summary(results: &[(String, usize)], platforms: &[&str]) -> Result<()> {
    let platform_counts = platforms.iter().map(|platform| filter_count(results, platform));
    let other_count = results.iter().map(|(_, count)| count).sum::<usize>()
        - platform_counts.clone().sum::<usize>();
    let width = std::cmp::max(
        // Length of "ansi_escape.rs"
        results.iter().map(|(file_name, _)| file_name.len()).max().unwrap_or_default(),
        // Length of "Total (windows)"
        platforms.iter().map(|s| s.len()).max().unwrap_or_default() + 8usize,
    );
    for (file_name, count) in results {
        println!("{file_name:width$} {count:4}");
    }
    let mut too_high = false;
    for (platform, platform_count) in platforms.iter().zip(platform_counts) {
        let header = format!("Total ({platform})");
        let total = platform_count + other_count;
        if total > 1024 {
            too_high = true;
            println!("{BOLD}{header:width$} {total:4}{RESET}  {RED}(> 1024){RESET}");
        } else {
            println!("{BOLD}{header:width$} {total:4}{RESET}  {GREEN}(≤ 1024){RESET}");
        }
    }
    if too_high {
        Err("Total count is too high")?;
    }
    Ok(())
}

fn filter_count(results: &[(String, usize)], platform: &str) -> usize {
    results
        .iter()
        .filter(|(file_name, _)| *file_name == format!("{platform}.rs"))
        .map(|(_, count)| count)
        .sum()
}
