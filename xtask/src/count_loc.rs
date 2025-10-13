use std::path::{Path, PathBuf};

use glob::glob;
use tokei::LanguageType;

use crate::{BOLD, GREEN, RED, RESET, Result};

const INNER_ATTRIBUTE_PREFIX: &str = "#![";
const OUTER_ATTRIBUTE_PREFIX: &str = "#[";
const ATTRIBUTE_PREFIXES_TO_IGNORE: [&str; 5] = [
    // Lint directives
    "allow(",
    "warn(",
    "deny(",
    "expect(",
    // Test-only attributes
    "cfg_attr(test,",
];

pub fn count_loc() -> Result<()> {
    let mut results = Vec::new();
    let config = tokei::Config::default();
    let mut longest_path = 0;
    for entry in glob("./*.rs")?.chain(glob("./src/**/*.rs")?) {
        let path = entry?;

        let source = filter_lines(&path)?;
        let stats = LanguageType::Rust.parse_from_str(source, &config);

        longest_path = longest_path.max(path.display().to_string().len());
        results.push((path, stats.code));
    }
    print_summary(&results, longest_path, &["unix", "wasi", "windows"])
}

/// Filter out lines that contain lints and anything after
/// `#[cfg(test)]` attributes.
pub fn filter_lines(path: &Path) -> Result<String> {
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

pub fn print_summary(results: &[(PathBuf, usize)], width: usize, platforms: &[&str]) -> Result<()> {
    let platform_counts = platforms.iter().map(|platform| filter_count(results, platform));
    let other_count = results.iter().map(|(_, count)| count).sum::<usize>()
        - platform_counts.clone().sum::<usize>();
    for (path, count) in results {
        println!("{:width$} {:4}", path.display(), count, width = width);
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

pub fn filter_count(results: &[(PathBuf, usize)], pattern: &str) -> usize {
    results
        .iter()
        .filter(|(path, _)| path.display().to_string().contains(pattern))
        .map(|(_, count)| count)
        .sum::<usize>()
}
