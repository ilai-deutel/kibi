//! # Kibi
//!
//! Kibi is a text editor in ≤1024 lines of code.

pub use crate::{config::Config, editor::Editor, error::Error};

pub mod ansi_escape;
mod config;
mod editor;
mod error;
mod row;
mod syntax;
mod terminal;

#[cfg(windows)] mod windows;
#[cfg(windows)] use windows as sys;

#[cfg(unix)] mod xdg;
#[cfg(unix)] mod unix;
#[cfg(unix)] use unix as sys;

#[cfg(target_os = "wasi")] mod xdg;
#[cfg(target_os = "wasi")] mod wasi;
#[cfg(target_os = "wasi")] use wasi as sys;
