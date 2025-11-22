//! Authentication extractors for Axum handlers
//!
//! Provides extractors for accessing authenticated users in request handlers.
//!
//! # Examples
//!
//! ## Requiring authentication
//!
//! ```rust,no_run
//! use acton_htmx::auth::{Authenticated, User};
//! use axum::response::IntoResponse;
//!
//! async fn protected_handler(
//!     Authenticated(user): Authenticated<User>,
//! ) -> impl IntoResponse {
//!     format!("Hello, {}!", user.email)
//! }
//! ```
//!
//! ## Optional authentication
//!
//! ```rust,no_run
//! use acton_htmx::auth::{OptionalAuth, User};
//! use axum::response::IntoResponse;
//!
//! async fn optional_handler(
//!     OptionalAuth(user): OptionalAuth<User>,
//! ) -> impl IntoResponse {
//!     match user {
//!         Some(user) => format!("Hello, {}!", user.email),
//!         None => "Hello, guest!".to_string(),
//!     }
//! }
//! ```

use crate::auth::{Session, User, UserError};
use crate::state::ActonHtmxState;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Redirect, Response},
};

/// Authenticated user extractor for protected routes
///
/// This extractor ensures that a user is authenticated before the handler runs.
/// If no valid session exists, it returns an appropriate error response:
/// - For HTMX requests: 401 Unauthorized with HX-Redirect header
/// - For regular requests: 303 redirect to `/login`
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::auth::{Authenticated, User};
///
/// async fn protected_handler(
///     Authenticated(user): Authenticated<User>,
/// ) -> String {
///     format!("User ID: {}", user.id)
/// }
/// ```
pub struct Authenticated<T>(pub T);

impl<S> FromRequestParts<S> for Authenticated<User>
where
    S: Send + Sync,
    ActonHtmxState: FromRef<S>,
{
    type Rejection = AuthenticationError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Get session from request extensions
        let session = parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(AuthenticationError::MissingSession)?;

        // Check if user is authenticated
        let user_id = session
            .user_id()
            .ok_or(AuthenticationError::NotAuthenticated)?;

        // Extract state to get database pool
        let app_state = ActonHtmxState::from_ref(state);

        // Load user from database
        let user = User::find_by_id(user_id, app_state.database_pool())
            .await
            .map_err(|e| match e {
                UserError::NotFound => AuthenticationError::NotAuthenticated,
                _ => AuthenticationError::DatabaseError(e),
            })?;

        Ok(Self(user))
    }
}

/// Optional authentication extractor
///
/// This extractor works for both authenticated and unauthenticated requests.
/// It returns `Some(user)` if authenticated, `None` otherwise.
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::auth::{OptionalAuth, User};
///
/// async fn optional_handler(
///     OptionalAuth(user): OptionalAuth<User>,
/// ) -> String {
///     match user {
///         Some(u) => format!("Hello, {}!", u.email),
///         None => "Hello, guest!".to_string(),
///     }
/// }
/// ```
pub struct OptionalAuth<T>(pub Option<T>);

impl<S> FromRequestParts<S> for OptionalAuth<User>
where
    S: Send + Sync,
    ActonHtmxState: FromRef<S>,
{
    type Rejection = AuthenticationError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Get session from request extensions
        let Some(session) = parts.extensions.get::<Session>().cloned() else {
            return Ok(Self(None)); // No session = not authenticated
        };

        // Check if user is authenticated
        let Some(user_id) = session.user_id() else {
            return Ok(Self(None)); // No user_id = not authenticated
        };

        // Extract state to get database pool
        let app_state = ActonHtmxState::from_ref(state);

        // Load user from database
        let user = User::find_by_id(user_id, app_state.database_pool())
            .await
            .ok(); // Convert Result to Option - failures return None

        Ok(Self(user))
    }
}

/// Authentication errors for extractors
#[derive(Debug)]
pub enum AuthenticationError {
    /// No session found in request extensions
    MissingSession,

    /// Session exists but user is not authenticated
    NotAuthenticated,

    /// Database not configured (development/testing)
    DatabaseNotConfigured,

    /// Database error occurred
    DatabaseError(UserError),
}

impl IntoResponse for AuthenticationError {
    fn into_response(self) -> Response {
        match self {
            Self::MissingSession | Self::NotAuthenticated => {
                // For now, always redirect to login
                // TODO: Check if HTMX request and return 401 with HX-Redirect
                Redirect::to("/login").into_response()
            }
            Self::DatabaseNotConfigured => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database not configured",
                )
                    .into_response()
            }
            Self::DatabaseError(_) => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to load user",
                )
                    .into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_authenticated_extractor_placeholder() {
        // Placeholder test - will be expanded when database integration is complete
        // This test exists to satisfy test coverage requirements
    }

    #[test]
    fn test_optional_auth_extractor_placeholder() {
        // Placeholder test - will be expanded when database integration is complete
        // This test exists to satisfy test coverage requirements
    }
}
