# Kibi: A text editor in ≤1024 lines of code, written in Rust

[![Build Status](https://travis-ci.com/ilai-deutel/kibi.svg?branch=master)](https://travis-ci.com/ilai-deutel/kibi)
[![Crate](https://img.shields.io/crates/v/kibi.svg)](https://crates.io/crates/kibi)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.41+-lightgray.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/crates/l/kibi)](#license)

[![asciicast](https://gist.githubusercontent.com/ilai-deutel/39670157dd008d9932b2f2fd3c885cca/raw/bfdbfc96181c4f6e3ce2663c25c6e97bf57c8684/kibi.gif)](https://asciinema.org/a/KY7tKPlxHXqRdJiv5KaTJbPj5)

A configurable text editor with incremental search, syntax highlighting, line numbers and more, written in less than 1024 lines<sup>[1](#counted-with)</sup> of Rust with minimal dependencies.

This project is inspired by [`kilo`](https://github.com/antirez/kilo), a text editor written in C, and [this tutorial](https://viewsourcecode.org/snaptoken/kilo/) (also in C).

Contributions are welcome! Be careful to stay below the 1024-line limit...

<sub><a name="counted-with">1.</a>: Counted with [`tokei`](https://github.com/XAMPPRocky/tokei)</sub>

## Installation

You can install Kibi with [`cargo`](https://github.com/rust-lang/cargo/):

```bash
$ cargo install kibi
```

Syntax highlighting configuration files are available in the [`syntax`](syntax) directory of this repository.
They need to be placed in one of the configuration directories mentioned in the [Configuration/Syntax Highlighting](#syntax-higlighting) section.

For instance:

```bash
$ cd ~/repos
$ git clone https://github.com/ilai-deutel/kibi.git
$ mkdir -p ~/.config/kibi
$ ln -sr ./kibi/syntax ~/.config/kibi/syntax.d
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

### Configuration

#### Global configuration

Kibi can be configured using:
* A system-wide configuration file, located at `/etc/kibi/config.ini`
* A user-level configuration file, located at:
  * `$XDG_CONFIG_HOME/kibi/config.ini` if environment variable `$XDG_CONFIG_HOME` is defined
  * `~/.config/kibi/config.ini` otherwise

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

#### Syntax Higlighting

Syntax highlighting can be configured using INI files located at:
* `/etc/kibi/syntax.d/<file_name>.ini` for system-wide availability
* For user-level configuration files:
  * `$XDG_CONFIG_HOME/kibi/syntax.d/<file_name>.ini` if environment variable `$XDG_CONFIG_HOME` is defined
  * `~/.config/kibi/syntax.d/<file_name>.ini` otherwise

Syntax highlighting configuration follows this format:

```ini
### /etc/kibi/syntax.d/rust.ini ###
# Kibi syntax highlighting configuration for Rust

name=Rust
extensions=rs
highlight_numbers=true
highlight_strings=true
singleline_comment_start=//
multiline_comment_delim=/*, */
; The keyword list is taken from here: https://doc.rust-lang.org/book/appendix-01-keywords.html
keywords_1=abstract, as, async, await, become, box, break, const, continue, crate, do, dyn, else, enum, extern, false, final, fn, for, if, impl, in, let, loop, macro, match, mod, move, mut, override, priv, pub, ref, return, self, Self, static, struct, super, trait, true, try, type, typeof, unsafe, unsized, use, virtual, where, while, yield
keywords_2=i8, i16, i32, i64, i128, isize, u8, u16, u32, u36, u128, usize, f32, f64, bool, char, str
```

## Dependencies

This project must remain tiny, so using advanced dependencies such as [`ncurses`](https://crates.io/crates/ncurses), [`toml`](https://crates.io/crates/toml) or [`ansi-escapes`](https://crates.io/crates/ansi-escapes) would be cheating.
The only dependencies provide safe wrappers around `libc` calls, to avoid using `unsafe` code as much as possible:

* `libc`
* `nix`
* `signal-hook`

## Why Kibi?

1. Porting the `kilo` source code from C to Rust and trying to make it idiomatic was interesting
2. Implementing new features while under the 1024-line constraint is a good challenge
3. Most importantly, I wanted to learn Rust and this was a great project to start (thanks Reddit for the idea)


## License

Kibi is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and [COPYRIGHT](COPYRIGHT) for details.
