# Changelog

## [0.2.3] - 2021-09-30

### Added
- Add syntax highlighting configuration for C ([#98](https://github.com/ilai-deutel/kibi/pull/98))
- Add syntax highlighting configuration for Nim ([#106](https://github.com/ilai-deutel/kibi/pull/106))
- Add syntax highlighting configuration for Lua ([#108](https://github.com/ilai-deutel/kibi/pull/108))
- Add syntax highlighting configuration for Javascript ([#108](https://github.com/ilai-deutel/kibi/pull/108))

### Updated
- Updated syntax highlighting configuration for V ([#108](https://github.com/ilai-deutel/kibi/pull/108))

## [0.2.2] - 2021-02-12

### Added
- Add syntax highlighting configuration for V ([#78](https://github.com/ilai-deutel/kibi/pull/78))
- Add the ability to execute external commands from the editor ([#83](https://github.com/ilai-deutel/kibi/pull/83))
- Improve file opening error messages for config files ([#91](https://github.com/ilai-deutel/kibi/pull/91))

### Fixed
- Android: fix a bug ([#87](https://github.com/ilai-deutel/kibi/issues/87)) related to a SELinux policy that would cause
  Kibi to crash on certain Android versions when setting the termios
  ([#92](https://github.com/ilai-deutel/kibi/pull/92)). 

## [0.2.1] - 2020-10-05

### Added

- Add syntax configuration `singleline_string_quotes`, which specifies the list
  of characters to consider as quote (e.g. `", '` for Rust, `"` for JSON) ([#46](https://github.com/ilai-deutel/kibi/pull/46))

### Changes

- Internal code changes to reduce the binary size, and remove dependencies `nix`
  and `signal-hooks` ([#48](https://github.com/ilai-deutel/kibi/pull/48),
  [#49](https://github.com/ilai-deutel/kibi/pull/49), [#50](https://github.com/ilai-deutel/kibi/pull/50))

### Removed

- Remove boolean syntax configuration `highlight_strings`; use
  `singleline_string_quotes` instead ([#46](https://github.com/ilai-deutel/kibi/pull/46))

## [0.2.0] - 2020-04-24

### Added

- Add support for Windows 10
  ([#26](https://github.com/ilai-deutel/kibi/issues/26),
  [#34](https://github.com/ilai-deutel/kibi/issues/34), [#36](https://github.com/ilai-deutel/kibi/issues/36))
- Add a `--version` argument to the binary ([#31](https://github.com/ilai-deutel/kibi/pull/31))

### Changed

- Simplify `Row::update_syntax()`
- Rename the `multiline_comment_delim` configuration field to `multiline_comment_delims`
- Implement the `Default` trait for `Config`
  ([#12](https://github.com/ilai-deutel/kibi/issues/12)), `Editor`
  ([#20](https://github.com/ilai-deutel/kibi/issues/20)), and the `Debug` trait
  for `Error` ([#35](https://github.com/ilai-deutel/kibi/issues/35))
- The _find_ command now searchs in `row.chars`, not `row.renders`. A _tab_ will
  no longer be matched when searching for a space
  ([#23](https://github.com/ilai-deutel/kibi/issues/23))
- Use the XDG base directory specification for configuration files
  (global configuration, syntax highlighting confiuration) ([#42](https://github.com/ilai-deutel/kibi/issues/42))

### Fixed

- Fix syntax higlighting issue when an empty line is inserted in the middle of a
  multi-line string or a multi-line comment ([#7](https://github.com/ilai-deutel/kibi/issues/7))
- Fix crash when inserting a new line in the middle of a row ([#13](https://github.com/ilai-deutel/kibi/issues/13))
- Fix comments ([#12](https://github.com/ilai-deutel/kibi/issues/12), [#17](https://github.com/ilai-deutel/kibi/issues/17))
- Fix row not being updated after pressing backspace; fix syntax highlighting
  updates when inserting a new line ([#15](https://github.com/ilai-deutel/kibi/issues/15))
- Fix clippy lint warnings
  ([#21](https://github.com/ilai-deutel/kibi/issues/21),
  [#42](https://github.com/ilai-deutel/kibi/issues/42), [#43](https://github.com/ilai-deutel/kibi/issues/43))
- Fix match highlight when UTF-8 characters are present in the row ([#18](https://github.com/ilai-deutel/kibi/issues/18))

## [0.1.2] - 2020-02-13

### Added

- Add support for UTF-8 characters ([#1](https://github.com/ilai-deutel/kibi/issues/1))
- Add a command to duplicate the current row (`Ctrl-D`)
- Add syntax highlighting configuration for `bash`

### Fixed

- Fix path for system-wide configuration file
- Fix final new line being omitted during `load()`
- Trim spaces in the extensions enumeration in the syntax config file
- Fix erroneous field in example configuration `config.ini`

## [0.1.1] - 2020-02-13

kibi v0.1.1 is a small patch release that includes a minor fix to the
[crates.io package metadata](https://crates.io/crates/kibi).

### Added

- Add a config file example

### Fixed

- Fix `Cargo.toml` metadata, in particular incorrect categories

## [0.1.0] - 2020-02-11 [YANKED]

Initial release

[Next release]: https://github.com/ilai-deutel/kibi/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/ilai-deutel/kibi/releases/tag/v0.2.3
[0.2.2]: https://github.com/ilai-deutel/kibi/releases/tag/v0.2.2
[0.2.1]: https://github.com/ilai-deutel/kibi/releases/tag/v0.2.1
[0.2.0]: https://github.com/ilai-deutel/kibi/releases/tag/v0.2.0
[0.1.2]: https://github.com/ilai-deutel/kibi/releases/tag/v0.1.2
[0.1.1]: https://github.com/ilai-deutel/kibi/releases/tag/v0.1.1
[0.1.0]: https://github.com/ilai-deutel/kibi/releases/tag/v0.1.0
