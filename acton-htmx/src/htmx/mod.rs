//! HTMX response types and extractors
//!
//! This module builds on `axum-htmx` with additional features:
//! - Out-of-band swaps (`HxSwapOob`)
//! - Automatic template detection (`HxTemplate`)
//! - Smart response enum (`HxResponse`)
//!
//! # Re-exported from axum-htmx
//!
//! All request extractors and response helpers from `axum-htmx` are re-exported
//! for convenience. See [axum-htmx documentation](https://docs.rs/axum-htmx) for
//! detailed usage.

#![allow(dead_code)]

// Re-export axum-htmx request extractors
pub use axum_htmx::{
    HxBoosted, HxCurrentUrl, HxHistoryRestoreRequest, HxPrompt, HxRequest, HxTarget, HxTrigger,
    HxTriggerName,
};

// Re-export axum-htmx response helpers
pub use axum_htmx::{
    HxLocation, HxPushUrl, HxRedirect, HxRefresh, HxReplaceUrl, HxReselect, HxResponseTrigger,
    HxReswap, HxRetarget,
};

// Re-export axum-htmx middleware and guards
pub use axum_htmx::{AutoVaryLayer, HxRequestGuardLayer};

// TODO: Implement acton-htmx extensions
// mod swap_oob;
// mod response;
// pub use swap_oob::HxSwapOob;
// pub use response::HxResponse;
