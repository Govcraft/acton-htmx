//! Testing utilities for acton-htmx applications
//!
//! This module provides test helpers for integration testing HTMX applications:
//! - [`TestServer`] - Wrapper around `axum-test` for server testing
//! - [`TestDatabase`] - Helper for SQLx test databases
//! - HTMX assertion helpers for common response patterns
//! - Re-exported mockall-generated mocks for traits
//!
//! # Example
//!
//! ```rust,no_run
//! use acton_htmx::testing::{TestServer, TestDatabase};
//! use acton_htmx::prelude::*;
//!
//! #[tokio::test]
//! async fn test_login_flow() {
//!     let app = build_test_app().await;
//!     let server = TestServer::new(app).unwrap();
//!
//!     let response = server
//!         .post("/login")
//!         .form(&LoginForm {
//!             email: "test@example.com",
//!             password: "password123",
//!         })
//!         .await;
//!
//!     server.assert_hx_redirect(&response, "/dashboard");
//! }
//! ```

pub mod assertions;
pub mod database;
pub mod server;

// Re-export for convenience
pub use assertions::*;
pub use database::TestDatabase;
pub use server::TestServer;

// Re-export mockall for test usage
pub use mockall;
