//! Acton CLI module
//!
//! Provides command-line interface for the Acton framework ecosystem.
//!
//! # Subcommands
//!
//! - `htmx` - HTMX web framework commands

pub mod htmx;

pub use htmx::{DatabaseBackend, HtmxCommand};
