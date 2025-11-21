//! acton-htmx CLI library

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![warn(clippy::cargo)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::multiple_crate_versions)]

pub mod templates;

pub use templates::ProjectTemplate;
