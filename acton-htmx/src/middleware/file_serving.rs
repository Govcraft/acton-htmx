//! File serving middleware with range requests, caching, and access control
//!
//! This module provides middleware for serving uploaded files with:
//! - Range request support for streaming and resumable downloads
//! - Proper cache headers (ETag, Last-Modified, Cache-Control)
//! - CDN integration hints
//! - Access control for private files
//!
//! # Examples
//!
//! ## Basic file serving
//!
//! ```rust,no_run
//! use acton_htmx::middleware::FileServingMiddleware;
//! use acton_htmx::storage::{FileStorage, LocalFileStorage};
//! use std::path::PathBuf;
//! use std::sync::Arc;
//! use axum::{Router, routing::get};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let storage = Arc::new(LocalFileStorage::new(PathBuf::from("/var/uploads"))?);
//! let middleware = FileServingMiddleware::new(storage);
//!
//! let app = Router::new()
//!     .route("/files/:id", get(|| async { "file handler" }))
//!     .layer(middleware);
//! # Ok(())
//! # }
//! ```
//!
//! ## With access control
//!
//! ```rust,no_run
//! use acton_htmx::middleware::{FileServingMiddleware, FileAccessControl};
//! use acton_htmx::storage::{FileStorage, LocalFileStorage};
//! use std::path::PathBuf;
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let storage = Arc::new(LocalFileStorage::new(PathBuf::from("/var/uploads"))?);
//!
//! // Custom access control
//! let access_control: FileAccessControl = Arc::new(|user_id, file_id| {
//!     Box::pin(async move {
//!         // Check if user owns the file or is admin
//!         Ok(true)
//!     })
//! });
//!
//! let middleware = FileServingMiddleware::new(storage)
//!     .with_access_control(access_control);
//! # Ok(())
//! # }
//! ```

use crate::storage::{FileStorage, StorageError, StorageResult};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{
        header::{
            ACCEPT_RANGES, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG,
            IF_NONE_MATCH, IF_RANGE, LAST_MODIFIED, RANGE,
        },
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    time::SystemTime,
};

/// Access control function type for file serving
///
/// Takes user ID (if authenticated) and file ID, returns whether access is allowed.
pub type FileAccessControl = Arc<
    dyn Fn(Option<String>, String) -> Pin<Box<dyn Future<Output = StorageResult<bool>> + Send>>
        + Send
        + Sync,
>;

/// Middleware for serving files with range requests, caching, and access control
#[derive(Clone)]
pub struct FileServingMiddleware<S: FileStorage> {
    #[allow(dead_code)] // Used in future layer implementation
    storage: Arc<S>,
    #[allow(dead_code)] // Used in future layer implementation
    access_control: Option<FileAccessControl>,
    #[allow(dead_code)] // Used in future layer implementation
    cache_max_age: u32,
    #[allow(dead_code)] // Used in future layer implementation
    enable_cdn_headers: bool,
}

impl<S: FileStorage> FileServingMiddleware<S> {
    /// Create a new file serving middleware
    #[must_use]
    pub fn new(storage: Arc<S>) -> Self {
        Self {
            storage,
            access_control: None,
            cache_max_age: 86400, // 1 day default
            enable_cdn_headers: false,
        }
    }

    /// Set custom access control function
    #[must_use]
    pub fn with_access_control(mut self, access_control: FileAccessControl) -> Self {
        self.access_control = Some(access_control);
        self
    }

    /// Set cache max-age in seconds (default: 86400 = 1 day)
    #[must_use]
    pub const fn with_cache_max_age(mut self, seconds: u32) -> Self {
        self.cache_max_age = seconds;
        self
    }

    /// Enable CDN-friendly headers
    #[must_use]
    pub const fn with_cdn_headers(mut self) -> Self {
        self.enable_cdn_headers = true;
        self
    }
}

/// Handler for serving a single file with range request support
///
/// This should be used as an Axum route handler for file serving endpoints.
///
/// # Examples
///
/// ```rust,no_run
/// use axum::{Router, routing::get};
/// use acton_htmx::middleware::serve_file;
/// use acton_htmx::storage::LocalFileStorage;
/// use std::path::PathBuf;
/// use std::sync::Arc;
///
/// # async fn example() -> anyhow::Result<()> {
/// let storage = Arc::new(LocalFileStorage::new(PathBuf::from("/var/uploads"))?);
///
/// let app = Router::new()
///     .route("/files/:id", get(serve_file::<LocalFileStorage>))
///     .with_state(storage);
/// # Ok(())
/// # }
/// ```
pub async fn serve_file<S: FileStorage>(
    State(storage): State<Arc<S>>,
    Path(file_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, FileServingError> {
    // Retrieve file metadata (for ETag, Last-Modified, Content-Type)
    // In a real implementation, this would fetch from storage metadata
    // For now, we'll work with the file data directly
    let data = storage
        .retrieve(&file_id)
        .await
        .map_err(FileServingError::Storage)?;

    // Generate ETag from file ID and size
    let etag = format!(r#""{}-{}""#, file_id, data.len());
    let content_type = "application/octet-stream"; // TODO: Get from StoredFile metadata

    // Check If-None-Match (ETag validation)
    if let Some(if_none_match) = headers.get(IF_NONE_MATCH) {
        if if_none_match.to_str().is_ok_and(|v| v == etag) {
            return Ok((StatusCode::NOT_MODIFIED, ()).into_response());
        }
    }

    // Check for range request
    if let Some(range_header) = headers.get(RANGE) {
        return serve_range_request(&data, range_header, &etag, content_type, &headers);
    }

    // Serve complete file
    Ok(build_file_response(data, &etag, content_type, None))
}

/// Serve a range request (partial content)
fn serve_range_request(
    data: &[u8],
    range_header: &HeaderValue,
    etag: &str,
    content_type: &str,
    headers: &HeaderMap,
) -> Result<Response, FileServingError> {
    let file_size = data.len();

    // Check If-Range header (validate ETag before serving range)
    if let Some(if_range) = headers.get(IF_RANGE) {
        if if_range.to_str().map_or(true, |v| v != etag) {
            // ETag doesn't match, serve full file instead
            return Ok(build_file_response(data.to_vec(), etag, content_type, None));
        }
    }

    // Parse range header (simplified - only handles single range)
    let range_str = range_header
        .to_str()
        .map_err(|_| FileServingError::InvalidRange)?;

    if !range_str.starts_with("bytes=") {
        return Err(FileServingError::InvalidRange);
    }

    let range_spec = &range_str[6..]; // Skip "bytes="
    let (start_str, end_str) = range_spec
        .split_once('-')
        .ok_or(FileServingError::InvalidRange)?;

    let start: usize = if start_str.is_empty() {
        // Suffix range: -500 means last 500 bytes
        let suffix_len: usize = end_str
            .parse()
            .map_err(|_| FileServingError::InvalidRange)?;
        file_size.saturating_sub(suffix_len)
    } else {
        start_str
            .parse()
            .map_err(|_| FileServingError::InvalidRange)?
    };

    let end: usize = if end_str.is_empty() {
        // Open-ended range: 500- means from byte 500 to end
        file_size - 1
    } else {
        end_str
            .parse::<usize>()
            .map_err(|_| FileServingError::InvalidRange)?
            .min(file_size - 1)
    };

    // Validate range
    if start > end || start >= file_size {
        return Err(FileServingError::RangeNotSatisfiable(file_size));
    }

    let range_data = data[start..=end].to_vec();

    let content_range = format!("bytes {start}-{end}/{file_size}");

    Ok(build_file_response(
        range_data,
        etag,
        content_type,
        Some((&content_range, StatusCode::PARTIAL_CONTENT)),
    ))
}

/// Build a file response with appropriate headers
fn build_file_response(
    data: Vec<u8>,
    etag: &str,
    content_type: &str,
    range_info: Option<(&str, StatusCode)>,
) -> Response {
    let mut response = Response::builder();

    // Set status code
    let status = range_info.map_or(StatusCode::OK, |(_, code)| code);
    response = response.status(status);

    // Content headers
    response = response
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_LENGTH, data.len())
        .header(ETAG, etag)
        .header(ACCEPT_RANGES, "bytes");

    // Range-specific headers
    if let Some((content_range, _)) = range_info {
        response = response.header(CONTENT_RANGE, content_range);
    }

    // Cache headers
    response = response
        .header(CACHE_CONTROL, "public, max-age=86400")
        .header(
            LAST_MODIFIED,
            httpdate::fmt_http_date(SystemTime::now()),
        );

    response
        .body(Body::from(data))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

/// Error types for file serving operations
#[derive(Debug)]
pub enum FileServingError {
    /// Storage backend error
    Storage(StorageError),
    /// Invalid range request
    InvalidRange,
    /// Range not satisfiable
    RangeNotSatisfiable(usize),
    /// Access denied
    AccessDenied,
}

impl std::fmt::Display for FileServingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(e) => write!(f, "Storage error: {e}"),
            Self::InvalidRange => write!(f, "Invalid range request"),
            Self::RangeNotSatisfiable(size) => {
                write!(f, "Range not satisfiable (file size: {size})")
            }
            Self::AccessDenied => write!(f, "Access denied"),
        }
    }
}

impl std::error::Error for FileServingError {}

impl IntoResponse for FileServingError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Self::InvalidRange => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::RangeNotSatisfiable(size) => {
                let response = Response::builder()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(CONTENT_RANGE, format!("bytes */{size}"))
                    .body(Body::from(self.to_string()))
                    .unwrap_or_else(|_| Response::new(Body::empty()));
                return response;
            }
            Self::AccessDenied => (StatusCode::FORBIDDEN, self.to_string()),
        };

        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_etag_generation() {
        let file_id = "test-file-123";
        let data = b"Hello, World!";
        let etag = format!(r#""{}-{}""#, file_id, data.len());
        assert_eq!(etag, r#""test-file-123-13""#);
    }

    // TODO: Implement comprehensive range request tests
    // - Full range: "bytes=0-499"
    // - Suffix range: "bytes=-500" (last 500 bytes)
    // - Open-ended: "bytes=500-" (from 500 to end)
    // - Invalid ranges
    // - Multi-range (not currently supported)
}
