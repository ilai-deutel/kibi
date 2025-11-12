# Kibi: A text editor in ≤1024 lines of code, written in Rust

[![Build Status](https://github.com/ilai-deutel/kibi/actions/workflows/ci.yml/badge.svg)](https://github.com/ilai-deutel/kibi/actions/workflows/ci.yml)
[![Lines of code](https://github.com/ilai-deutel/kibi/actions/workflows/loc.yml/badge.svg)](https://github.com/ilai-deutel/kibi/actions/workflows/loc.yml)
[![CodeQL](https://github.com/ilai-deutel/kibi/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/ilai-deutel/kibi/actions/workflows/github-code-scanning/codeql)
[![Crate](https://img.shields.io/crates/v/kibi.svg)](https://crates.io/crates/kibi)
[![Platform](https://img.shields.io/badge/platform-Linux%20|%20macOS%20|%20Windows%2010%20|%20WASI-blue)](#)
[![Packaging status](https://repology.org/badge/tiny-repos/kibi.svg)](https://repology.org/project/kibi/versions)
[![Dependency Status](https://deps.rs/repo/github/ilai-deutel/kibi/status.svg)](https://deps.rs/repo/github/ilai-deutel/kibi)
[![Coverage Status](https://coveralls.io/repos/github/ilai-deutel/kibi/badge.svg)](https://coveralls.io/github/ilai-deutel/kibi)
[![License](https://img.shields.io/crates/l/kibi?color=blue)](#license)
[![REUSE status](https://api.reuse.software/badge/github.com/ilai-deutel/kibi)](https://api.reuse.software/info/github.com/ilai-deutel/kibi)
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/ilai-deutel/kibi)](https://rust-reportcard.xuri.me/report/github.com/ilai-deutel/kibi)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/ilai-deutel/kibi/badge)](https://scorecard.dev/viewer/?uri=github.com/ilai-deutel/kibi)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/11296/badge)](https://www.bestpractices.dev/projects/11296)
[![SLSA 3](https://slsa.dev/images/gh-badge-level3.svg)](https://slsa.dev)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-3.0-5E0D73.svg?logo=contributorcovenant)](CODE_OF_CONDUCT.md)<!-- ALL-CONTRIBUTORS-BADGE:START - Do not remove or modify this section -->
[![All Contributors](https://img.shields.io/badge/all_contributors-40-orange.svg)](#contributors)
<!-- ALL-CONTRIBUTORS-BADGE:END -->

[![asciicast](assets/asciicast.gif)](https://asciinema.org/a/KY7tKPlxHXqRdJiv5KaTJbPj5)

A configurable text editor with UTF-8 support, incremental search, syntax
highlighting, line numbers and more, written in less than 1024
lines[^1] of Rust with minimal dependencies.

[^1]: Counted per platform, excluding tests and Clippy directives, see [`count_loc`](./xtask/src/count_loc.rs)

Kibi is compatible with Linux, macOS, Windows
10[^2], and WASI.

[^2]: Kibi requires the terminal to support ANSI escape sequences. Windows 10 version 1703 ([Creators Update](https://devblogs.microsoft.com/commandline/windows-10-creators-update-whats-new-in-bashwsl-windows-console), April 2017) and above are supported.</sub>

This project is inspired by [`kilo`](https://github.com/antirez/kilo), a text
editor written in C.
See [comparison](#comparison-with-kilo) below for a list of additional features.

Contributions are welcome! Be careful to stay below the 1024-line limit...

## Table of contents

* [Installation](#installation)
  * [With cargo](#with-cargo)
  * [Arch User Repository (Arch Linux)](#arch-user-repository-arch-linux)
  * [Android (with Termux)](#android-with-termux)
  * [Fedora/CentOS](#fedoracentos)
  * [NetBSD](#netbsd)
  * [Flatpak](#flatpak)
* [Usage](#usage)
  * [Keyboard shortcuts](#keyboard-shortcuts)
  * [Configuration](#configuration)
    * [Global configuration](#global-configuration)
    * [Syntax highlighting](#syntax-highlighting)
* [Comparison with kilo](#comparison-with-kilo)
* [Contributing](#contributing)
* [Dependencies](#dependencies)
* [Why Kibi?](#why-kibi)
* [Contributors](#contributors)
* [License](#license)

## Installation

<!-- markdownlint-disable MD033 -->
<a href="https://repology.org/project/kibi/versions">
    <img src="https://repology.org/badge/vertical-allrepos/kibi.svg"
    alt="Packaging status" align="right">
</a>
<!-- markdownlint-enable MD033 -->

### With `cargo`

You can install Kibi with [`cargo`](https://github.com/rust-lang/cargo/):

```bash
cargo install kibi
```

Syntax highlighting configuration files are available in the
[`syntax.d`](syntax.d) directory of this repository. They need to be placed in
one of the configuration directories mentioned in the
[Configuration/Syntax Highlighting](#syntax-highlighting) section.

For instance:

```bash
cd ~/repos
git clone https://github.com/ilai-deutel/kibi.git
mkdir -p ~/.local/share/kibi/
ln -sr ./kibi/syntax.d ~/.local/share/kibi/syntax.d
```

### Arch User Repository (Arch Linux)

2 packages are available on the AUR:
[`kibi`](https://aur.archlinux.org/packages/kibi) and
[`kibi-git`](https://aur.archlinux.org/packages/kibi-git).

<details><summary>Installation instructions</summary>

1. Installation with an AUR helper, for instance using `yay`:

   ```bash
   yay -Syu kibi  # or yay -Syu kibi-git
   ```

2. Install manually with `makepkg`:

   ```bash
   git clone https://aur.archlinux.org/kibi.git  # or git clone https://aur.archlinux.org/kibi-git.git
   cd kibi
   makepkg -si
   ```

</details>

### Android (with Termux)

Kibi is available as [a Termux package](https://github.com/termux/termux-packages/tree/master/packages/kibi).

<details><summary>Installation instructions</summary>

1. Install Termux:

   [![Get it on F-Droid"](https://user-images.githubusercontent.com/10098207/80576418-fe805600-89b9-11ea-82a5-7dc00c0ccc50.png)](https://f-droid.org/repository/browse/?fdid=com.termux)
   [![Get it on Google Play"](https://user-images.githubusercontent.com/10098207/80576344-e27cb480-89b9-11ea-9fa6-08ec94af10cb.png)](https://play.google.com/store/apps/details?id=com.termux)

2. Install Kibi within Termux: `pkg install kibi`

</details>

### Fedora/CentOS

The package is available in COPR as [`atim/kibi`](https://copr.fedorainfracloud.org/coprs/atim/kibi).

<details><summary>Installation instructions</summary>

```bash
sudo dnf copr enable atim/kibi -y
sudo dnf install kibi
```

</details>

### NetBSD

Kibi is available from [the official repos](https://cdn.netbsd.org/pub/pkgsrc/current/pkgsrc/editors/kibi/README.html).

<details><summary>Installation instructions</summary>

Install using:

```bash
pkgin install kibi
```

or build from source:

```bash
cd /usr/pkgsrc/editors/kibi
make install
```

</details>

### Flatpak

Kibi is available [on Flathub](https://flathub.org/en/apps/com.github.ilai_deutel.kibi).

<details><summary>Command line instructions</summary>
 The flatpak can be installed using:

```bash
flatpak install flathub com.github.ilai_deutel.kibi
```

You can then run Kibi with:

```bash
flatpak run com.github.ilai_deutel.kibi
```

</details>

## Usage

```bash
kibi              # Start an new text buffer
kibi <file path>  # Open a file
kibi --version    # Print version information and exit
```

### Keyboard shortcuts

| Keyboard shortcut              | Description                                                   |
| ------------------------------ | ------------------------------------------------------------- |
| <kbd>Ctrl</kbd> + <kbd>F</kbd> | Incremental search; use arrows to navigate                    |
| <kbd>Ctrl</kbd> + <kbd>S</kbd> | Save the buffer to the current file, or specify the file path |
| <kbd>Ctrl</kbd> + <kbd>G</kbd> | Go to `<line number>[:<column number>]` position              |
| <kbd>Ctrl</kbd> + <kbd>Q</kbd> | Quit                                                          |
| <kbd>Ctrl</kbd> + <kbd>D</kbd> | Duplicate the current row                                     |
| <kbd>Ctrl</kbd> + <kbd>E</kbd> | Execute an external command and paste its output              |
| <kbd>Ctrl</kbd> + <kbd>R</kbd> | Remove an entire line                                         |
| <kbd>Ctrl</kbd> + <kbd>C</kbd> | Copies the entire line                                        |
| <kbd>Ctrl</kbd> + <kbd>X</kbd> | Cuts the entire line                                          |
| <kbd>Ctrl</kbd> + <kbd>V</kbd> | Will paste the copied line                                    |
| <kbd>Ctrl</kbd> + <kbd>←</kbd> | Moves cursor to previous word                                 |
| <kbd>Ctrl</kbd> + <kbd>→</kbd> | Moves cursor to next word                                     |

### Configuration

#### Global configuration

Kibi can be configured using a configuration file. It must follow this format:

```ini
# The size of a tab. Must be > 0.
tab_stop=4
# The number of confirmations needed before quitting, when changes have been
# made since the file was last changed.
quit_times=2
# The duration for which messages are shown in the status bar, in seconds.
message_duration=3
# Whether to show line numbers.
show_line_numbers=true
```

The location of these files is described below.

##### Linux / macOS

kibi follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html):

* A user-level configuration file can be located at
  `$XDG_CONFIG_HOME/kibi/config.ini` if environment variable `$XDG_CONFIG_HOME`
  is defined, `~/.config/kibi/config.ini` otherwise.
* A system-wide configuration file can be located at
  `$XDG_CONFIG_DIRS/kibi/config.ini` if environment variable
  `$XDG_CONFIG_DIRS` is defined, `/etc/kibi/config.ini` or
  `/etc/xdg/kibi/config.ini` otherwise.

##### Windows

A configuration file can be located at `%APPDATA%\Kibi\config.ini`.

#### Syntax highlighting

Syntax highlighting can be configured using INI files which follow this format:

<!-- markdownlint-disable MD013 -->
```ini
### /usr/share/kibi/syntax.d/rust.ini ###
# Kibi syntax highlighting configuration for Rust

name=Rust
extensions=rs
highlight_numbers=true
singleline_string_quotes="
singleline_comment_start=//
multiline_comment_delims=/*, */
; In Rust, the multi-line string delimiter is the same as the single-line string
; delimiter
multiline_string_delim="
; https://doc.rust-lang.org/book/appendix-01-keywords.html
keywords_1=abstract, as, async, await, become, box, break, const, continue, crate, do, dyn, else, enum, extern, false, final, fn, for, if, impl, in, let, loop, macro, match, mod, move, mut, override, priv, pub, ref, return, self, Self, static, struct, super, trait, true, try, type, typeof, unsafe, unsized, use, virtual, where, while, yield
keywords_2=i8, i16, i32, i64, i128, isize, u8, u16, u32, u36, u128, usize, f32, f64, bool, char, str
```
<!-- markdownlint-enable MD013 -->

The location of these files is described below.

##### Linux / macOS

kibi follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html):

* User-level syntax highlighting configuration files can be located at `$XDG_DATA_HOME/kibi/syntax.d/<file_name>.ini`
  if environment variable `$XDG_DATA_HOME` is defined,
  `~/.local/share/kibi/syntax.d/<file_name>.ini` otherwise.
* System-wide syntax highlighting configuration files can be located at `$XDG_DATA_DIRS/kibi/syntax.d/<file_name>.ini`
  if environment variable `$XDG_DATA_DIRS` is defined,
  `/usr/local/share/kibi/syntax.d/<file_name>.ini` or
  `/usr/share/kibi/syntax.d/<file_name>.ini` otherwise.

##### Windows

Syntax highlighting configuration files can be located at `%APPDATA%\Kibi\syntax.d\<file_name>.ini`.

## Comparison with `kilo`

This project is inspired by [`kilo`](https://github.com/antirez/kilo), a text
editor written by Salvatore Sanfilippo (antirez) in C, and
[this tutorial](https://viewsourcecode.org/snaptoken/kilo/) (also in C).

`kibi` provides additional features:

* Support for UTF-8 characters
* Compatibility with Windows
* Command to jump to a given row/column
* Handle window resize (UNIX only)
* Parsing configuration files: global editor configuration with validation, language-specific
  syntax highlighting configuration (38 languages and counting)
* Display line numbers on the left of the screen; display file size in the status
  bar
* Syntax highlighting: multi-line strings
* *Save as* prompt when no file name has been provided
* Command to duplicate the current row, to quickly move between words
* Ability to execute an external command from the editor and paste its output
* Support for end-of-options delimiter `--` (following [POSIX.1-2024 12. Utility Conventions](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap12.html))
* Can be used as the editor for the *`sudoers`* file via [`visudo`](https://www.man7.org/linux/man-pages/man8/visudo.8.html)
* Guaranteed memory safety, thanks to Rust!
* Bug fixes

## Contributing

The most important limitation is that the line check the line count remains
under 1024. To check this run:

```shell
cargo xtask count-loc
```

## Dependencies

This project must remain tiny, so using advanced dependencies such as [`ncurses`](https://crates.io/crates/ncurses),
[`toml`](https://crates.io/crates/toml) or
[`ansi-escapes`](https://crates.io/crates/ansi-escapes) would be cheating.

The following dependencies provide wrappers around system calls.

* On UNIX systems (Linux, macOS):
  * `libc`
* On Windows:
  * `winapi`
  * `winapi-util`

In addition, `unicode-width` is used to determine the displayed width of Unicode
characters. Unfortunately, there is no way around it: the
[unicode character width table](https://github.com/unicode-rs/unicode-width/blob/3033826f8bf05e82724140a981d5941e48fce393/src/tables.rs#L52)
is 230 lines long.

## Why Kibi?

1. Porting the `kilo` source code from C to Rust and trying to make it idiomatic
   was interesting
2. Implementing new features while under the 1024-line constraint is a good challenge
3. Most importantly, I wanted to learn Rust and this was a great project to
   start (thanks Reddit for the idea)

## Contributors

This project follows the [all-contributors](https://allcontributors.org/) specification
([emoji key](https://allcontributors.org/docs/en/emoji-key)). Contributions of
any kind welcome!

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ilai-deutel"><img src="https://avatars0.githubusercontent.com/u/10098207?v=4" width="100px;" alt=""/><br /><sub><b>Ilaï Deutel</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=ilai-deutel" title="Code">💻</a> <a href="#ideas-ilai-deutel" title="Ideas, Planning, & Feedback">🤔</a> <a href="#infra-ilai-deutel" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#platform-ilai-deutel" title="Packaging/porting to new platform">📦</a> <a href="#syntax-highlighting-ilai-deutel" title="Contributing to syntax highlighting configuration files">💠</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/codingonHP"><img src="https://avatars3.githubusercontent.com/u/7105293?v=4" width="100px;" alt=""/><br /><sub><b>Vishal Anand</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/issues?q=author%3AcodingonHP" title="Bug reports">🐛</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/gsbhasin123"><img src="https://avatars2.githubusercontent.com/u/48339289?v=4" width="100px;" alt=""/><br /><sub><b>Gurjus Bhasin</b></sub></a><br /><a href="#ideas-gsbhasin123" title="Ideas, Planning, & Feedback">🤔</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ldang0"><img src="https://avatars1.githubusercontent.com/u/1281722?v=4" width="100px;" alt=""/><br /><sub><b>Libin Dang</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=ldang0" title="Code">💻</a> <a href="https://github.com/ilai-deutel/kibi/issues?q=author%3Aldang0" title="Bug reports">🐛</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/andradei"><img src="https://avatars0.githubusercontent.com/u/2653546?v=4" width="100px;" alt=""/><br /><sub><b>Isaac Andrade</b></sub></a><br /><a href="#ideas-andradei" title="Ideas, Planning, & Feedback">🤔</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/frjol"><img src="https://avatars2.githubusercontent.com/u/4422813?v=4" width="100px;" alt=""/><br /><sub><b>frjol</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/issues?q=author%3Afrjol" title="Bug reports">🐛</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/tim77"><img src="https://avatars0.githubusercontent.com/u/5614476?v=4" width="100px;" alt=""/><br /><sub><b>Artem Polishchuk</b></sub></a><br /><a href="#platform-tim77" title="Packaging/porting to new platform">📦</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/malbarbo"><img src="https://avatars3.githubusercontent.com/u/1678126?v=4" width="100px;" alt=""/><br /><sub><b>Marco A L Barbosa</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=malbarbo" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/voidpin"><img src="https://avatars3.githubusercontent.com/u/35956139?v=4" width="100px;" alt=""/><br /><sub><b>pin</b></sub></a><br /><a href="#platform-voidpin" title="Packaging/porting to new platform">📦</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Uniminin"><img src="https://avatars3.githubusercontent.com/u/42061747?v=4" width="100px;" alt=""/><br /><sub><b>Uniminin</b></sub></a><br /><a href="#syntax-highlighting-Uniminin" title="Contributing to syntax highlighting configuration files">💠</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Woomy4680-exe"><img src="https://avatars.githubusercontent.com/u/57042741?v=4" width="100px;" alt=""/><br /><sub><b>Woomy4680-exe</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/issues?q=author%3AWoomy4680-exe" title="Bug reports">🐛</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/quadroli"><img src="https://avatars.githubusercontent.com/u/58091631?v=4" width="100px;" alt=""/><br /><sub><b>quadroli</b></sub></a><br /><a href="#syntax-highlighting-quadroli" title="Contributing to syntax highlighting configuration files">💠</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/m0lese"><img src="https://avatars.githubusercontent.com/u/86180257?v=4" width="100px;" alt=""/><br /><sub><b>molese</b></sub></a><br /><a href="#syntax-highlighting-m0lese" title="Contributing to syntax highlighting configuration files">💠</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/imreyesjorge"><img src="https://avatars.githubusercontent.com/u/40706868?v=4" width="100px;" alt=""/><br /><sub><b>Jorge Reyes</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=imreyesjorge" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/gVirtu"><img src="https://avatars.githubusercontent.com/u/15658199?v=4" width="100px;" alt=""/><br /><sub><b>Giancarlo França</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=gVirtu" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ilmanzo"><img src="https://avatars.githubusercontent.com/u/1872816?v=4" width="100px;" alt=""/><br /><sub><b>Andrea Manzini</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=ilmanzo" title="Code">💻</a> <a href="https://github.com/ilai-deutel/kibi/commits?author=ilmanzo" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Arkaeriit"><img src="https://avatars.githubusercontent.com/u/47985708?v=4" width="100px;" alt=""/><br /><sub><b>Maxime Bouillot</b></sub></a><br /><a href="#syntax-highlighting-Arkaeriit" title="Contributing to syntax highlighting configuration files">💠</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/john-sharratt"><img src="https://avatars.githubusercontent.com/u/17711537?v=4" width="100px;" alt=""/><br /><sub><b>Johnathan Sharratt</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=john-sharratt" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/No-one-important"><img src="https://avatars.githubusercontent.com/u/72458263?v=4" width="100px;" alt=""/><br /><sub><b>No-one-important</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=No-one-important" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/RobertGrancsa"><img src="https://avatars.githubusercontent.com/u/22158239?v=4" width="100px;" alt=""/><br /><sub><b>Robert Grancsa</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=RobertGrancsa" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/sogreatandpowerfull"><img src="https://avatars.githubusercontent.com/u/114095654?v=4" width="100px;" alt=""/><br /><sub><b>SoGreatAndPowerful</b></sub></a><br /><a href="#syntax-highlighting-sogreatandpowerfull" title="Contributing to syntax highlighting configuration files">💠</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/carol-mb"><img src="https://avatars.githubusercontent.com/u/117681554?v=4" width="100px;" alt=""/><br /><sub><b>Mihai-Carol Bazga</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=carol-mb" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/NickGeek"><img src="https://avatars.githubusercontent.com/u/3462055?v=4" width="100px;" alt=""/><br /><sub><b>Nick Webster</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=NickGeek" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/hezhizhen"><img src="https://avatars.githubusercontent.com/u/7611700?v=4" width="100px;" alt=""/><br /><sub><b>Zhizhen He</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=hezhizhen" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/CosminGGeorgescu"><img src="https://avatars.githubusercontent.com/u/95082750?v=4" width="100px;" alt=""/><br /><sub><b>CosminGGeorgescu</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=CosminGGeorgescu" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/TanvirOnGH"><img src="https://avatars.githubusercontent.com/u/138336161?v=4" width="100px;" alt=""/><br /><sub><b>Tanvir</b></sub></a><br /><a href="#syntax-highlighting-TanvirOnGH" title="Contributing to syntax highlighting configuration files">💠</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/BogdanPaul15"><img src="https://avatars.githubusercontent.com/u/62438221?v=4" width="100px;" alt=""/><br /><sub><b>Prisacaru Bogdan-Paul</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=BogdanPaul15" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/auzkok"><img src="https://avatars.githubusercontent.com/u/35302680?v=4" width="100px;" alt=""/><br /><sub><b>auzkok</b></sub></a><br /><a href="#syntax-highlighting-auzkok" title="Contributing to syntax highlighting configuration files">💠</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/razvanrus2003"><img src="https://avatars.githubusercontent.com/u/69893593?v=4" width="100px;" alt=""/><br /><sub><b>Razvan Rus</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=razvanrus2003" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Jan9103"><img src="https://avatars.githubusercontent.com/u/55753387?v=4" width="100px;" alt=""/><br /><sub><b>Jan9103</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=Jan9103" title="Code">💻</a> <a href="#syntax-highlighting-Jan9103" title="Contributing to syntax highlighting configuration files">💠</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/joshka"><img src="https://avatars.githubusercontent.com/u/381361?v=4" width="100px;" alt=""/><br /><sub><b>Josh McKinney</b></sub></a><br /><a href="#infra-joshka" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="https://github.com/ilai-deutel/kibi/commits?author=joshka" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/zamazan4ik"><img src="https://avatars.githubusercontent.com/u/7355383?v=4" width="100px;" alt=""/><br /><sub><b>Alexander Zaitsev</b></sub></a><br /><a href="#infra-zamazan4ik" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/BanuAdrian"><img src="https://avatars.githubusercontent.com/u/66678108?v=4" width="100px;" alt=""/><br /><sub><b>Adrian Banu</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=BanuAdrian" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/vanillajonathan"><img src="https://avatars.githubusercontent.com/u/10222521?v=4" width="100px;" alt=""/><br /><sub><b>Jonathan</b></sub></a><br /><a href="#syntax-highlighting-vanillajonathan" title="Contributing to syntax highlighting configuration files">💠</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/lythd"><img src="https://avatars.githubusercontent.com/u/22566088?v=4" width="100px;" alt=""/><br /><sub><b>ly</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=lythd" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/kianmeng"><img src="https://avatars.githubusercontent.com/u/134518?v=4" width="100px;" alt=""/><br /><sub><b>Kian-Meng Ang</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=kianmeng" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/iamdhakrey"><img src="https://avatars.githubusercontent.com/u/35266081?v=4" width="100px;" alt=""/><br /><sub><b>Hrithik Dhakrey</b></sub></a><br /><a href="https://github.com/ilai-deutel/kibi/commits?author=iamdhakrey" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/kenderipa"><img src="https://avatars.githubusercontent.com/u/52325831?v=4" width="100px;" alt=""/><br /><sub><b>kenderipa</b></sub></a><br /><a href="#ideas-kenderipa" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/ilai-deutel/kibi/issues?q=author%3Akenderipa" title="Bug reports">🐛</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ahaoboy"><img src="https://avatars.githubusercontent.com/u/19884146?v=4" width="100px;" alt=""/><br /><sub><b>阿豪</b></sub></a><br /><a href="#infra-ahaoboy" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/luizvbo"><img src="https://avatars.githubusercontent.com/u/15856439?v=4" width="100px;" alt=""/><br /><sub><b>Luiz Otavio Vilas Boas Oliveira</b></sub></a><br /><a href="#infra-luizvbo" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

This project is licensed under either of

* [Apache License, Version 2.0](LICENSE-APACHE)
* [MIT License](LICENSE-MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Kibi by you shall be dual licensed as above, without any
additional terms or conditions.
