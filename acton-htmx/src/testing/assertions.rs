//! HTMX-specific assertion helpers for testing
//!
//! This module provides helper functions for asserting HTMX response headers
//! and behavior in integration tests.

use axum_test::TestResponse;

/// Assert that the response contains an HX-Redirect header with the expected path
///
/// # Panics
///
/// Panics if the header is missing or has a different value
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::testing::{TestServer, assert_hx_redirect};
///
/// # async fn example() {
/// # let server = todo!();
/// let response = server.post("/login").await;
/// assert_hx_redirect(&response, "/dashboard");
/// # }
/// ```
pub fn assert_hx_redirect(response: &TestResponse, expected_path: &str) {
    let header = response
        .headers()
        .get("HX-Redirect")
        .expect("HX-Redirect header not found");
    let actual = header.to_str().expect("Invalid HX-Redirect header value");
    assert_eq!(
        actual, expected_path,
        "Expected HX-Redirect to {expected_path}, got {actual}"
    );
}

/// Assert that the response contains an HX-Trigger header with the expected event
///
/// # Panics
///
/// Panics if the header is missing or doesn't contain the event
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::testing::{TestServer, assert_hx_trigger};
///
/// # async fn example() {
/// # let server = todo!();
/// let response = server.post("/create").await;
/// assert_hx_trigger(&response, "itemCreated");
/// # }
/// ```
pub fn assert_hx_trigger(response: &TestResponse, expected_event: &str) {
    let header = response
        .headers()
        .get("HX-Trigger")
        .expect("HX-Trigger header not found");
    let actual = header.to_str().expect("Invalid HX-Trigger header value");
    assert!(
        actual.contains(expected_event),
        "Expected HX-Trigger to contain '{expected_event}', got '{actual}'"
    );
}

/// Assert that the response contains an HX-Reswap header with the expected swap strategy
///
/// # Panics
///
/// Panics if the header is missing or has a different value
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::testing::{TestServer, assert_hx_reswap};
///
/// # async fn example() {
/// # let server = todo!();
/// let response = server.get("/content").await;
/// assert_hx_reswap(&response, "outerHTML");
/// # }
/// ```
pub fn assert_hx_reswap(response: &TestResponse, expected_swap: &str) {
    let header = response
        .headers()
        .get("HX-Reswap")
        .expect("HX-Reswap header not found");
    let actual = header.to_str().expect("Invalid HX-Reswap header value");
    assert_eq!(
        actual, expected_swap,
        "Expected HX-Reswap to be {expected_swap}, got {actual}"
    );
}

/// Assert that the response contains an HX-Retarget header with the expected target
///
/// # Panics
///
/// Panics if the header is missing or has a different value
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::testing::{TestServer, assert_hx_retarget};
///
/// # async fn example() {
/// # let server = todo!();
/// let response = server.get("/content").await;
/// assert_hx_retarget(&response, "#main");
/// # }
/// ```
pub fn assert_hx_retarget(response: &TestResponse, expected_target: &str) {
    let header = response
        .headers()
        .get("HX-Retarget")
        .expect("HX-Retarget header not found");
    let actual = header.to_str().expect("Invalid HX-Retarget header value");
    assert_eq!(
        actual, expected_target,
        "Expected HX-Retarget to be {expected_target}, got {actual}"
    );
}

/// Assert that the response contains an HX-Push-Url header
///
/// # Panics
///
/// Panics if the header is missing
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::testing::{TestServer, assert_hx_push_url};
///
/// # async fn example() {
/// # let server = todo!();
/// let response = server.post("/create").await;
/// assert_hx_push_url(&response, Some("/items/123"));
/// # }
/// ```
pub fn assert_hx_push_url(response: &TestResponse, expected_url: Option<&str>) {
    let header = response
        .headers()
        .get("HX-Push-Url")
        .expect("HX-Push-Url header not found");
    let actual = header.to_str().expect("Invalid HX-Push-Url header value");

    if let Some(url) = expected_url {
        assert_eq!(
            actual, url,
            "Expected HX-Push-Url to be {url}, got {actual}"
        );
    } else {
        assert_eq!(
            actual, "false",
            "Expected HX-Push-Url to be false, got {actual}"
        );
    }
}

// Tests for assertion helpers require integration tests with actual TestServer
// See integration tests in tests/ directory for usage examples
