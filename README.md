# Kibi: A text editor in ≤1024 lines of code, written in Rust

[![Build Status](https://img.shields.io/travis/com/ilai-deutel/kibi/master?logo=travis)](https://travis-ci.com/ilai-deutel/kibi)
[![Crate](https://img.shields.io/crates/v/kibi.svg)](https://crates.io/crates/kibi)
[![AUR](https://img.shields.io/aur/version/kibi.svg?logo=arch-linux)](https://aur.archlinux.org/packages/kibi/)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.41+-blue.svg?logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows%2010-blue)](#)
[![License](https://img.shields.io/crates/l/kibi?color=blue)](#license)

[![asciicast](https://gist.githubusercontent.com/ilai-deutel/39670157dd008d9932b2f2fd3c885cca/raw/bfdbfc96181c4f6e3ce2663c25c6e97bf57c8684/kibi.gif)](https://asciinema.org/a/KY7tKPlxHXqRdJiv5KaTJbPj5)

A configurable text editor with UTF-8 support, incremental search, syntax highlighting, line numbers and more, written
in less than 1024 lines<sup>[1](#counted-with)</sup> of Rust with minimal dependencies.

Kibi is compatible with Linux, macOS, and Windows 10 (beta).

This project is inspired by [`kilo`](https://github.com/antirez/kilo), a text editor written in C.
See [comparison](#comparison-with-kilo) below for a list of additional features.

Contributions are welcome! Be careful to stay below the 1024-line limit...

<sub><a name="counted-with">1.</a>: Counted with [`tokei`](https://github.com/XAMPPRocky/tokei)</sub>

## Installation

### With `cargo`

You can install Kibi with [`cargo`](https://github.com/rust-lang/cargo/):

```bash
$ cargo install kibi
```

Syntax highlighting configuration files are available in the [`config_example/syntax.d`](config_example/syntax.d)
directory of this repository. They need to be placed in one of the configuration directories mentioned in the
[Configuration/Syntax Highlighting](#syntax-highlighting) section.

For instance:

```bash
$ cd ~/repos
$ git clone https://github.com/ilai-deutel/kibi.git
$ mkdir -p ~/.config/kibi
$ ln -sr ./kibi/syntax ~/.config/kibi/syntax.d
```

### Arch User Repository (Arch Linux)

[`kibi`](https://aur.archlinux.org/packages/kibi/) is available on the AUR.

#### Installation with an AUR helper

For instance, using `yay`: 

```bash
yay -Syu kibi
```

#### Manual installation with `makepkg`

```bash
git clone https://aur.archlinux.org/kibi.git
cd kibi
makepkg -si
```

## Usage

```bash
# Start an new text buffer
$ kibi
# Open a file
$ kibi <file path>
```

### Keyboard shortcuts

| Keyboard shortcut | Description                                                   |
| ----------------- | ------------------------------------------------------------- |
| Ctrl-F            | Incremental search; use arrows to navigate                    |
| Ctrl-S            | Save the buffer to the current file, or specify the file path |
| Ctrl-G            | Go to `<line number>[:<column number>]` position              |
| Ctrl-Q            | Quit                                                          |
| Ctrl-D            | Duplicate the current row                                     |

### Configuration

#### Global configuration

Kibi can be configured using configuration files. The location of these files is described below.

* Linux / macOS:
    * `/etc/kibi/config.ini` (system-wide configuration file)
    * A user-level configuration file can be located located at:
      * `$XDG_CONFIG_HOME/kibi/config.ini` if environment variable `$XDG_CONFIG_HOME` is defined
      * `~/.config/kibi/config.ini` otherwise
* Windows:
    * `%APPDATA%\kibi\config.ini`

Example configuration file:
```ini
# The size of a tab. Must be > 0.
tab_stop=4
# The number of confirmations needed before quitting, when changes have been made since the file.
# was last changed.
quit_times=2
# The duration for which messages are shown in the status bar, in seconds.
message_duration=3
# Whether to show line numbers.
show_line_numbers=true
```

#### Syntax Highlighting

Syntax highlighting can be configured using INI files located at:

* Linux / macOS:
    * `/etc/kibi/syntax.d/<file_name>.ini` for system-wide availability
    * For user-level configuration files:
      * `$XDG_CONFIG_HOME/kibi/syntax.d/<file_name>.ini` if environment variable `$XDG_CONFIG_HOME` is defined
      * `~/.config/kibi/syntax.d/<file_name>.ini` otherwise
* Windows:
    * `%APPDATA%\kibi\syntax.d\<file_name>.ini`

Syntax highlighting configuration follows this format:

```ini
### /etc/kibi/syntax.d/rust.ini ###
# Kibi syntax highlighting configuration for Rust

name=Rust
extensions=rs
highlight_numbers=true
highlight_strings=true
singleline_comment_start=//
multiline_comment_delims=/*, */
; In Rust, the multi-line string delimiter is the same as the single-line string delimiter
multiline_string_delim="
; https://doc.rust-lang.org/book/appendix-01-keywords.html
keywords_1=abstract, as, async, await, become, box, break, const, continue, crate, do, dyn, else, enum, extern, false, final, fn, for, if, impl, in, let, loop, macro, match, mod, move, mut, override, priv, pub, ref, return, self, Self, static, struct, super, trait, true, try, type, typeof, unsafe, unsized, use, virtual, where, while, yield
keywords_2=i8, i16, i32, i64, i128, isize, u8, u16, u32, u36, u128, usize, f32, f64, bool, char, str
```

## Comparison with `kilo`

This project is inspired by [`kilo`](https://github.com/antirez/kilo), a text editor written by Salvatore Sanfilippo
(antirez) in C, and [this tutorial](https://viewsourcecode.org/snaptoken/kilo/) (also in C).

`kibi` provides additional features:
- Support for UTF-8 characters
- Compatible with Windows
- Command to jump to a given row/column
- Handle window resize (UNIX only)
- Parsing configuration files: global editor configuration, language-specific syntax highlighting configuration
- Display line numbers on the left of the screen; display file size in the status bar
- Syntax highlighting: multi-line strings
- *Save as* prompt when no file name has been provided
- Command to duplicate the current row
- Memory safety, thanks to Rust!
- Many bug fixes

## Dependencies

This project must remain tiny, so using advanced dependencies such as [`ncurses`](https://crates.io/crates/ncurses),
[`toml`](https://crates.io/crates/toml) or [`ansi-escapes`](https://crates.io/crates/ansi-escapes) would be cheating.

The following dependencies provide wrappers around system calls. Safe wrappers are preferred to avoid using `unsafe` code as much as possible:

* On UNIX systems (Linux, macOS):
    * `libc`
    * `nix`
    * `signal-hook`
* On Windows:
    * `winapi`
    * `winapi-util`

In addition, `unicode-width` is used to determine the displayed width of Unicode characters. Unfortunately, there is no
way around it: the [unicode character width table](https://github.com/unicode-rs/unicode-width/blob/3033826f8bf05e82724140a981d5941e48fce393/src/tables.rs#L52)
is 230 lines long.

## Why Kibi?

1. Porting the `kilo` source code from C to Rust and trying to make it idiomatic was interesting
2. Implementing new features while under the 1024-line constraint is a good challenge
3. Most importantly, I wanted to learn Rust and this was a great project to start (thanks Reddit for the idea)

## License

Kibi is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and [COPYRIGHT](COPYRIGHT) for details.
