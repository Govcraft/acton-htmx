//! Axum extractors for acton-htmx
//!
//! Provides extractors for accessing session data, flash messages,
//! CSRF tokens, validation, and other request context within handlers.

mod csrf;
mod session;
mod validated;

pub use csrf::CsrfTokenExtractor;
pub use session::{FlashExtractor, OptionalSession, SessionExtractor};
pub use validated::{
    format_validation_errors, validation_errors_json, ValidatedForm, ValidationError,
};
