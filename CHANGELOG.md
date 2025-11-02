# Changelog

All notable changes to this project will be documented in this file.

_The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)._

## [0.3.1] - 2025-11-01

### Added

- Support for end-of-options delimiter `--` (following [POSIX.1-2024 12. Utility Conventions](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap12.html)), allowing Kibi to be used as the editor for the *`sudoers`* file with [`visudo`](https://www.man7.org/linux/man-pages/man8/visudo.8.html) ([#481](https://github.com/ilai-deutel/kibi/pull/481))

### Fixed

- Crash when backspace was pressed in find mode with empty search field ([#482](https://github.com/ilai-deutel/kibi/pull/482))

## [0.3.0] - 2025-10-26

### Added

- Delete line with Ctrl-R ([#114](https://github.com/ilai-deutel/kibi/pull/114))
- Copy, cut and paste lines with Ctrl-C, Ctrl-X, Ctrl-V ([#207](https://github.com/ilai-deutel/kibi/pull/207))
- Ctrl+arrows moves to the previous/next word ([#214](https://github.com/ilai-deutel/kibi/pull/214))
- Syntax highlighting configurations for C ([#98](https://github.com/ilai-deutel/kibi/pull/98), [#181](https://github.com/ilai-deutel/kibi/pull/181)), Nim ([#106](https://github.com/ilai-deutel/kibi/pull/106)), C# ([#211](https://github.com/ilai-deutel/kibi/pull/211)), C++ ([#211](https://github.com/ilai-deutel/kibi/pull/211)), CoffeeScript ([#262](https://github.com/ilai-deutel/kibi/pull/262)), CSS ([#211](https://github.com/ilai-deutel/kibi/pull/211)), D ([#262](https://github.com/ilai-deutel/kibi/pull/262)), Dart ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Elixir ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Fish ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Go ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Groovy ([#262](https://github.com/ilai-deutel/kibi/pull/262)), Haskell ([#211](https://github.com/ilai-deutel/kibi/pull/211)), HTNL ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Java ([#211](https://github.com/ilai-deutel/kibi/pull/211)), JavaScript ([#108](https://github.com/ilai-deutel/kibi/pull/108)), Julia ([#262](https://github.com/ilai-deutel/kibi/pull/262)), Kotlin ([#211](https://github.com/ilai-deutel/kibi/pull/211)), LRC [#433](https://github.com/ilai-deutel/kibi/pull/433), Lua ([#108](https://github.com/ilai-deutel/kibi/pull/108), [#277](https://github.com/ilai-deutel/kibi/pull/277)), Markdown ([#152](https://github.com/ilai-deutel/kibi/pull/152)), MATLAB ([#262](https://github.com/ilai-deutel/kibi/pull/262)), Nix ([#262](https://github.com/ilai-deutel/kibi/pull/262)), NoSQL ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Nushell ([#262](https://github.com/ilai-deutel/kibi/pull/262), [#433](https://github.com/ilai-deutel/kibi/pull/433)), OCaml ([#262](https://github.com/ilai-deutel/kibi/pull/262)), Perl ([#211](https://github.com/ilai-deutel/kibi/pull/211)), PHP ([#211](https://github.com/ilai-deutel/kibi/pull/211)), PowerShell ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Processing ([#262](https://github.com/ilai-deutel/kibi/pull/262)), PRQL ([#369](https://github.com/ilai-deutel/kibi/pull/369)), R ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Racket ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Ruby ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Raku ([#262](https://github.com/ilai-deutel/kibi/pull/262)), RSS [#433](https://github.com/ilai-deutel/kibi/pull/433), Scala ([#211](https://github.com/ilai-deutel/kibi/pull/211)), SQL ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Swift ([#211](https://github.com/ilai-deutel/kibi/pull/211)), TypeScript ([#211](https://github.com/ilai-deutel/kibi/pull/211)), XML ([#211](https://github.com/ilai-deutel/kibi/pull/211), [#449](https://github.com/ilai-deutel/kibi/pull/449)), YAML ([#211](https://github.com/ilai-deutel/kibi/pull/211)), Zig ([#262](https://github.com/ilai-deutel/kibi/pull/262), [#400](https://github.com/ilai-deutel/kibi/pull/400)), ZSH ([#211](https://github.com/ilai-deutel/kibi/pull/211))
- Support for WebAssembly ([#159](https://github.com/ilai-deutel/kibi/pull/159))
- Binary optimization for release: enable LTO ([#346](https://github.com/ilai-deutel/kibi/pull/346)); single codegen unit, abort on panic, strip symbols ([#464](https://github.com/ilai-deutel/kibi/pull/464))
- `kibi --version` includes git revision when available ([#176](https://github.com/ilai-deutel/kibi/pull/176))
- Extension-less dotfiles can now have a syntax highlight configuration ([#449](https://github.com/ilai-deutel/kibi/pull/449))
- Minimum Supported Rust Version (MSRV) in `Cargo.toml` ([#122](https://github.com/ilai-deutel/kibi/pull/122), [#133](https://github.com/ilai-deutel/kibi/pull/133), [#175](https://github.com/ilai-deutel/kibi/pull/175), [#191](https://github.com/ilai-deutel/kibi/pull/191), [#306](https://github.com/ilai-deutel/kibi/pull/306), [#307](https://github.com/ilai-deutel/kibi/pull/307), [#343](https://github.com/ilai-deutel/kibi/pull/343), [#442](https://github.com/ilai-deutel/kibi/pull/442))

### Changed

- Syntax highlighting configuration for V ([#108](https://github.com/ilai-deutel/kibi/pull/108))
- Use alternate screen buffer to avoid flicking, restore the terminal content on exit ([#310](https://github.com/ilai-deutel/kibi/pull/310))
- Various no-op code changes to reduce line count ([#127](https://github.com/ilai-deutel/kibi/pull/127), [#151](https://github.com/ilai-deutel/kibi/pull/151), [#154](https://github.com/ilai-deutel/kibi/pull/154), [#191](https://github.com/ilai-deutel/kibi/pull/191), [#229](https://github.com/ilai-deutel/kibi/pull/229), [#280](https://github.com/ilai-deutel/kibi/pull/280), [#335](https://github.com/ilai-deutel/kibi/pull/335), [#331](https://github.com/ilai-deutel/kibi/pull/331), [#330](https://github.com/ilai-deutel/kibi/pull/330), [#422](https://github.com/ilai-deutel/kibi/pull/422)) and to fix Clippy warnings ([#175](https://github.com/ilai-deutel/kibi/pull/175), [#188](https://github.com/ilai-deutel/kibi/pull/188), [#190](https://github.com/ilai-deutel/kibi/pull/190), [#206](https://github.com/ilai-deutel/kibi/pull/206), [#241](https://github.com/ilai-deutel/kibi/pull/241), [#249](https://github.com/ilai-deutel/kibi/pull/249), [#321](https://github.com/ilai-deutel/kibi/pull/321), [#334](https://github.com/ilai-deutel/kibi/pull/334), [#345](https://github.com/ilai-deutel/kibi/pull/345), [#385](https://github.com/ilai-deutel/kibi/pull/385))
- Rust edition: 2024 ([#442](https://github.com/ilai-deutel/kibi/pull/442))

### Fixed

- Crash when opening a new file ([#287](https://github.com/ilai-deutel/kibi/pull/287))
- Error message when an invalid option is provided ([#150](https://github.com/ilai-deutel/kibi/pull/150))
- Error message when trying to open a special file, e.g. UNIX devices or directories ([#159](https://github.com/ilai-deutel/kibi/pull/159))
- Config parsing with invalid durations ([#340](https://github.com/ilai-deutel/kibi/pull/340)), invalid tab size ([#450](https://github.com/ilai-deutel/kibi/pull/450))
- Emit warnings instead of panicking for invalid configurations ([#449](https://github.com/ilai-deutel/kibi/pull/449))

## [0.2.2] - 2021-02-12

### Added

- Syntax highlighting configuration for V ([#78](https://github.com/ilai-deutel/kibi/pull/78))
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
- The _find_ command now searches in `row.chars`, not `row.renders`. A _tab_ will
  no longer be matched when searching for a space
  ([#23](https://github.com/ilai-deutel/kibi/issues/23))
- Use the XDG base directory specification for configuration files
  (global configuration, syntax highlighting configuration) ([#42](https://github.com/ilai-deutel/kibi/issues/42))

### Fixed

- Fix syntax highlighting issue when an empty line is inserted in the middle of a
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
- Syntax highlighting configuration for `bash`

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

## [0.1.0] - 2020-02-11 _\[YANKED\]_

Initial release

[0.3.1]: https://github.com/ilai-deutel/kibi/releases/tag/v0.3.1
[0.3.0]: https://github.com/ilai-deutel/kibi/releases/tag/v0.3.0
[0.2.2]: https://github.com/ilai-deutel/kibi/releases/tag/v0.2.2
[0.2.1]: https://github.com/ilai-deutel/kibi/releases/tag/v0.2.1
[0.2.0]: https://github.com/ilai-deutel/kibi/releases/tag/v0.2.0
[0.1.2]: https://github.com/ilai-deutel/kibi/releases/tag/v0.1.2
[0.1.1]: https://github.com/ilai-deutel/kibi/releases/tag/v0.1.1
[0.1.0]: https://github.com/ilai-deutel/kibi/releases/tag/v0.1.0
