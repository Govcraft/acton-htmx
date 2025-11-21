//! Validated form extractor
//!
//! Provides automatic form validation using the validator crate.
//!
//! # Example
//!
//! ```rust,no_run
//! use acton_htmx::extractors::ValidatedForm;
//! use axum::response::Html;
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[derive(Debug, Deserialize, Validate)]
//! struct LoginForm {
//!     #[validate(email)]
//!     email: String,
//!     #[validate(length(min = 8))]
//!     password: String,
//! }
//!
//! async fn login(ValidatedForm(form): ValidatedForm<LoginForm>) -> Html<String> {
//!     // form is guaranteed to be valid here
//!     Html(format!("Logged in as {}", form.email))
//! }
//! ```

use axum::{
    extract::{Form, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use std::fmt;
use validator::Validate;

/// Validated form extractor
///
/// Automatically deserializes and validates form data using the validator crate.
/// Returns 422 Unprocessable Entity with validation errors if validation fails.
///
/// # Type Parameters
///
/// - `T`: The form type, must implement `Deserialize` and `Validate`
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::extractors::ValidatedForm;
/// use axum::response::Html;
/// use serde::Deserialize;
/// use validator::Validate;
///
/// #[derive(Debug, Deserialize, Validate)]
/// struct SignupForm {
///     #[validate(email)]
///     email: String,
///     #[validate(length(min = 8, max = 100))]
///     password: String,
///     #[validate(must_match(other = "password"))]
///     password_confirmation: String,
/// }
///
/// async fn signup(ValidatedForm(form): ValidatedForm<SignupForm>) -> Html<String> {
///     // form is guaranteed to be valid here
///     Html(format!("Signed up as {}", form.email))
/// }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate + 'static,
    S: Send + Sync + 'static,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Extract form data using standard Form extractor
        let Form(data) = Form::<T>::from_request(req, state)
            .await
            .map_err(|err| {
                ValidationError::FormRejection(format!("Failed to parse form data: {err}"))
            })?;

        // Validate the data
        data.validate()
            .map_err(ValidationError::Validation)?;

        Ok(Self(data))
    }
}

/// Validation error response
///
/// Returned when form validation fails. Contains detailed error information
/// about which fields failed validation and why.
#[derive(Debug)]
pub enum ValidationError {
    /// Form parsing failed (malformed data)
    FormRejection(String),
    /// Validation failed (data parsed but invalid)
    Validation(validator::ValidationErrors),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FormRejection(msg) => write!(f, "Form parsing error: {msg}"),
            Self::Validation(errors) => {
                write!(f, "Validation failed: ")?;
                for (field, errors) in errors.field_errors() {
                    write!(f, "{field}: ")?;
                    for error in errors {
                        if let Some(message) = &error.message {
                            write!(f, "{message}, ")?;
                        } else {
                            write!(f, "{}, ", error.code)?;
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        match self {
            Self::FormRejection(msg) => {
                (StatusCode::BAD_REQUEST, format!("Invalid form data: {msg}")).into_response()
            }
            Self::Validation(errors) => {
                let error_messages = format_validation_errors(&errors);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("Validation failed:\n{error_messages}"),
                )
                    .into_response()
            }
        }
    }
}

/// Format validation errors for display
///
/// Converts validator::ValidationErrors into a human-readable format.
///
/// # Example
///
/// ```rust
/// use validator::{Validate, ValidationErrors};
/// use acton_htmx::extractors::format_validation_errors;
///
/// let errors = ValidationErrors::new();
/// let formatted = format_validation_errors(&errors);
/// ```
#[must_use]
pub fn format_validation_errors(errors: &validator::ValidationErrors) -> String {
    let mut messages = Vec::new();

    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = error.message.as_ref().map_or_else(
                || format!("{field}: {}", error.code),
                ToString::to_string,
            );
            messages.push(message);
        }
    }

    messages.join("\n")
}

/// Validation error as JSON for HTMX responses
///
/// Returns validation errors in a structured JSON format suitable for HTMX
/// out-of-band swaps or client-side rendering.
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::extractors::{ValidatedForm, validation_errors_json};
/// use axum::response::{IntoResponse, Json};
/// use serde::Deserialize;
/// use validator::Validate;
///
/// #[derive(Debug, Deserialize, Validate)]
/// struct LoginForm {
///     #[validate(email)]
///     email: String,
/// }
///
/// async fn login(form: Result<ValidatedForm<LoginForm>, acton_htmx::extractors::ValidationError>) -> impl IntoResponse {
///     match form {
///         Ok(ValidatedForm(form)) => Json(serde_json::json!({"success": true})),
///         Err(err) => {
///             if let acton_htmx::extractors::ValidationError::Validation(errors) = err {
///                 return Json(validation_errors_json(&errors));
///             }
///             Json(serde_json::json!({"error": "Invalid form data"}))
///         }
///     }
/// }
/// ```
#[must_use]
pub fn validation_errors_json(errors: &validator::ValidationErrors) -> serde_json::Value {
    let mut error_map = serde_json::Map::new();

    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<String> = field_errors
            .iter()
            .map(|error| {
                error.message.as_ref().map_or_else(
                    || error.code.to_string(),
                    ToString::to_string,
                )
            })
            .collect();

        error_map.insert(field.to_string(), serde_json::json!(messages));
    }

    serde_json::json!({
        "errors": error_map
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        routing::post,
        Router,
    };
    use serde::Deserialize;
    use tower::ServiceExt;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    struct TestForm {
        #[validate(email)]
        email: String,
        #[validate(length(min = 8))]
        password: String,
    }

    async fn test_handler(ValidatedForm(form): ValidatedForm<TestForm>) -> String {
        format!("Email: {}, Password length: {}", form.email, form.password.len())
    }

    #[tokio::test]
    async fn test_valid_form() {
        let app = Router::new().route("/", post(test_handler));

        let request = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("email=test@example.com&password=password123"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_email() {
        let app = Router::new().route("/", post(test_handler));

        let request = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("email=invalid-email&password=password123"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_short_password() {
        let app = Router::new().route("/", post(test_handler));

        let request = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("email=test@example.com&password=short"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn test_format_validation_errors() {
        let mut errors = validator::ValidationErrors::new();
        errors.add(
            "email",
            validator::ValidationError::new("email")
                .with_message(std::borrow::Cow::Borrowed("Invalid email address")),
        );

        let formatted = format_validation_errors(&errors);
        assert!(formatted.contains("Invalid email address"));
    }

    #[test]
    fn test_validation_errors_json() {
        let mut errors = validator::ValidationErrors::new();
        errors.add(
            "email",
            validator::ValidationError::new("email")
                .with_message(std::borrow::Cow::Borrowed("Invalid email address")),
        );
        errors.add(
            "password",
            validator::ValidationError::new("length")
                .with_message(std::borrow::Cow::Borrowed("Password too short")),
        );

        let json = validation_errors_json(&errors);
        assert!(json.get("errors").is_some());

        let errors_obj = json.get("errors").unwrap().as_object().unwrap();
        assert!(errors_obj.contains_key("email"));
        assert!(errors_obj.contains_key("password"));
    }

    #[test]
    fn test_validation_error_display() {
        let mut errors = validator::ValidationErrors::new();
        errors.add(
            "email",
            validator::ValidationError::new("email")
                .with_message(std::borrow::Cow::Borrowed("Invalid email")),
        );

        let error = ValidationError::Validation(errors);
        let display = format!("{error}");
        assert!(display.contains("Validation failed"));
        assert!(display.contains("email"));
    }
}
