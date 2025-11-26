//! OAuth2 provider implementations
//!
//! This module contains implementations for various OAuth2 providers:
//! - Google OAuth2 (with OpenID Connect)
//! - GitHub OAuth2
//! - Generic OpenID Connect provider
//!
//! All providers use a shared `BaseOAuthProvider` to eliminate code duplication.

pub mod base;
pub mod github;
pub mod google;
pub mod oidc;

pub use base::BaseOAuthProvider;
pub use github::GitHubProvider;
pub use google::GoogleProvider;
pub use oidc::OidcProvider;
