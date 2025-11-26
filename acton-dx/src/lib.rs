//! Acton DX - Developer experience focused web framework for Rust
//!
//! This crate provides:
//! - **htmx**: HTMX web framework module (feature-gated)
//! - **cli**: Command-line interface module (feature-gated)
//!
//! # Features
//!
//! - `htmx` - HTMX web framework (default)
//! - `cli` - Command-line interface (default)
//! - `postgres` - PostgreSQL database support (default)
//! - `sqlite` - SQLite database support
//! - `mysql` - MySQL database support
//! - `redis` - Redis session and cache support (default)
//! - `cedar` - Cedar policy-based authorization (default)
//! - `otel-metrics` - OpenTelemetry metrics collection
//! - `aws-ses` - AWS SES email backend
//! - `clamav` - ClamAV virus scanning
//!
//! # Quick Start
//!
//! ## Using the CLI
//!
//! ```bash
//! # Create a new HTMX project
//! acton-dx htmx new my-app
//!
//! # Start development server
//! acton-dx htmx dev
//!
//! # Generate CRUD scaffold
//! acton-dx htmx scaffold crud Post title:string content:text
//! ```
//!
//! ## Using the Library
//!
//! ```rust,ignore
//! use acton_dx::htmx::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Your HTMX application code here
//!     Ok(())
//! }
//! ```

// Lint configuration is handled at the workspace level in Cargo.toml

/// HTMX web framework module
///
/// Contains all functionality for building server-rendered HTMX applications:
/// - Authentication and sessions
/// - Form handling with validation
/// - HTMX response types
/// - Middleware (CSRF, sessions, security headers)
/// - Email sending
/// - File storage
/// - Background jobs
/// - OAuth2 authentication
#[cfg(feature = "htmx")]
pub mod htmx;

/// Command-line interface module
///
/// Contains the CLI commands for:
/// - Project scaffolding (`acton htmx new`)
/// - Development server (`acton htmx dev`)
/// - Database management (`acton htmx db`)
/// - CRUD generation (`acton htmx scaffold`)
/// - Template management (`acton htmx templates`)
/// - Deployment (`acton htmx deploy`)
#[cfg(feature = "cli")]
pub mod cli;
