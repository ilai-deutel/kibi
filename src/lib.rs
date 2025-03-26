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

#[cfg_attr(windows, path = "windows.rs")]
#[cfg_attr(unix, path = "unix.rs")]
#[cfg_attr(target_os = "wasi", path = "wasi.rs")]
mod sys;

#[cfg(any(unix, target_os = "wasi"))] mod xdg;
