//! Upload policy system for fine-grained access control
//!
//! This module provides a flexible policy system for controlling file uploads
//! based on user roles, file types, quotas, and other constraints.
//!
//! # Examples
//!
//! ```rust
//! use acton_htmx::storage::policy::{UploadPolicy, PolicyBuilder};
//!
//! // Create a policy for regular users
//! let policy = PolicyBuilder::new()
//!     .max_file_size(10 * 1024 * 1024) // 10MB per file
//!     .allowed_mime_types(vec!["image/jpeg", "image/png", "application/pdf"])
//!     .max_total_storage(100 * 1024 * 1024) // 100MB total
//!     .build();
//!
//! // Check if an upload is allowed
//! let file_size = 5 * 1024 * 1024; // 5MB
//! let mime_type = "image/jpeg";
//! let current_usage = 50 * 1024 * 1024; // 50MB used
//!
//! assert!(policy.allows_upload(file_size, mime_type, current_usage).is_ok());
//! ```

use super::types::{StorageError, StorageResult};

/// Upload policy that defines constraints on file uploads
///
/// Policies can be used to implement role-based upload restrictions,
/// file type filtering, quota enforcement, and rate limiting.
#[derive(Debug, Clone)]
pub struct UploadPolicy {
    /// Maximum file size in bytes (None = unlimited)
    max_file_size: Option<u64>,

    /// Allowed MIME types (None = all allowed)
    allowed_mime_types: Option<Vec<String>>,

    /// Maximum total storage in bytes (None = unlimited)
    max_total_storage: Option<u64>,

    /// Maximum uploads per time window (None = unlimited)
    max_uploads_per_window: Option<usize>,

    /// Time window duration in seconds for rate limiting
    rate_limit_window_secs: Option<u64>,
}

impl Default for UploadPolicy {
    /// Creates a permissive default policy with reasonable limits
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::UploadPolicy;
    ///
    /// let policy = UploadPolicy::default();
    /// ```
    fn default() -> Self {
        Self {
            max_file_size: Some(10 * 1024 * 1024), // 10MB
            allowed_mime_types: None,              // All types allowed
            max_total_storage: Some(1024 * 1024 * 1024), // 1GB
            max_uploads_per_window: Some(100),     // 100 uploads
            rate_limit_window_secs: Some(3600),    // Per hour
        }
    }
}

impl UploadPolicy {
    /// Creates a new policy builder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::UploadPolicy;
    ///
    /// let policy = UploadPolicy::builder()
    ///     .max_file_size(5 * 1024 * 1024)
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> PolicyBuilder {
        PolicyBuilder::new()
    }

    /// Creates an unrestricted policy (use with caution!)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::UploadPolicy;
    ///
    /// let policy = UploadPolicy::unrestricted();
    /// ```
    #[must_use]
    pub const fn unrestricted() -> Self {
        Self {
            max_file_size: None,
            allowed_mime_types: None,
            max_total_storage: None,
            max_uploads_per_window: None,
            rate_limit_window_secs: None,
        }
    }

    /// Creates a restrictive policy for untrusted users
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::UploadPolicy;
    ///
    /// let policy = UploadPolicy::restrictive();
    /// ```
    #[must_use]
    pub fn restrictive() -> Self {
        Self {
            max_file_size: Some(1024 * 1024), // 1MB
            allowed_mime_types: Some(vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
            ]),
            max_total_storage: Some(10 * 1024 * 1024), // 10MB
            max_uploads_per_window: Some(10),          // 10 uploads
            rate_limit_window_secs: Some(3600),        // Per hour
        }
    }

    /// Checks if an upload is allowed under this policy
    ///
    /// # Arguments
    ///
    /// * `file_size` - Size of the file to upload
    /// * `mime_type` - MIME type of the file
    /// * `current_storage_used` - Current storage usage in bytes
    ///
    /// # Errors
    ///
    /// Returns error if the upload violates policy constraints
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::UploadPolicy;
    ///
    /// let policy = UploadPolicy::default();
    /// let result = policy.allows_upload(5_000_000, "image/jpeg", 50_000_000);
    /// assert!(result.is_ok());
    /// ```
    pub fn allows_upload(
        &self,
        file_size: u64,
        mime_type: &str,
        current_storage_used: u64,
    ) -> StorageResult<()> {
        // Check file size limit
        if let Some(max_size) = self.max_file_size {
            if file_size > max_size {
                return Err(StorageError::FileSizeExceeded {
                    actual: file_size,
                    limit: max_size,
                });
            }
        }

        // Check MIME type allowlist
        if let Some(allowed_types) = &self.allowed_mime_types {
            if !allowed_types.iter().any(|t| t == mime_type) {
                return Err(StorageError::InvalidMimeType {
                    expected: allowed_types.clone(),
                    actual: mime_type.to_string(),
                });
            }
        }

        // Check total storage quota
        if let Some(max_storage) = self.max_total_storage {
            if current_storage_used + file_size > max_storage {
                return Err(StorageError::QuotaExceeded);
            }
        }

        Ok(())
    }

    /// Returns the maximum allowed file size
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::UploadPolicy;
    ///
    /// let policy = UploadPolicy::default();
    /// assert!(policy.max_file_size().is_some());
    /// ```
    #[must_use]
    pub const fn max_file_size(&self) -> Option<u64> {
        self.max_file_size
    }

    /// Returns the allowed MIME types
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::UploadPolicy;
    ///
    /// let policy = UploadPolicy::restrictive();
    /// assert!(policy.allowed_mime_types().is_some());
    /// ```
    #[must_use]
    pub fn allowed_mime_types(&self) -> Option<&[String]> {
        self.allowed_mime_types.as_deref()
    }

    /// Returns the maximum total storage quota
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::UploadPolicy;
    ///
    /// let policy = UploadPolicy::default();
    /// assert!(policy.max_total_storage().is_some());
    /// ```
    #[must_use]
    pub const fn max_total_storage(&self) -> Option<u64> {
        self.max_total_storage
    }

    /// Returns the rate limit configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::UploadPolicy;
    ///
    /// let policy = UploadPolicy::default();
    /// assert!(policy.rate_limit().is_some());
    /// ```
    #[must_use]
    pub const fn rate_limit(&self) -> Option<(usize, u64)> {
        match (self.max_uploads_per_window, self.rate_limit_window_secs) {
            (Some(max_uploads), Some(window_secs)) => Some((max_uploads, window_secs)),
            _ => None,
        }
    }
}

/// Builder for creating upload policies
///
/// # Examples
///
/// ```rust
/// use acton_htmx::storage::policy::PolicyBuilder;
///
/// let policy = PolicyBuilder::new()
///     .max_file_size(10 * 1024 * 1024)
///     .allowed_mime_types(vec!["image/jpeg", "image/png"])
///     .max_total_storage(100 * 1024 * 1024)
///     .rate_limit(100, 3600)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct PolicyBuilder {
    max_file_size: Option<u64>,
    allowed_mime_types: Option<Vec<String>>,
    max_total_storage: Option<u64>,
    max_uploads_per_window: Option<usize>,
    rate_limit_window_secs: Option<u64>,
}

impl PolicyBuilder {
    /// Creates a new policy builder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::PolicyBuilder;
    ///
    /// let builder = PolicyBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum file size in bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::PolicyBuilder;
    ///
    /// let policy = PolicyBuilder::new()
    ///     .max_file_size(10 * 1024 * 1024) // 10MB
    ///     .build();
    /// ```
    #[must_use]
    pub const fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    /// Sets the allowed MIME types
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::PolicyBuilder;
    ///
    /// let policy = PolicyBuilder::new()
    ///     .allowed_mime_types(vec!["image/jpeg", "image/png"])
    ///     .build();
    /// ```
    #[must_use]
    pub fn allowed_mime_types<I, S>(mut self, types: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_mime_types = Some(types.into_iter().map(Into::into).collect());
        self
    }

    /// Sets the maximum total storage quota in bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::PolicyBuilder;
    ///
    /// let policy = PolicyBuilder::new()
    ///     .max_total_storage(100 * 1024 * 1024) // 100MB
    ///     .build();
    /// ```
    #[must_use]
    pub const fn max_total_storage(mut self, size: u64) -> Self {
        self.max_total_storage = Some(size);
        self
    }

    /// Sets the rate limit (max uploads per time window)
    ///
    /// # Arguments
    ///
    /// * `max_uploads` - Maximum number of uploads allowed
    /// * `window_secs` - Time window in seconds
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::PolicyBuilder;
    ///
    /// let policy = PolicyBuilder::new()
    ///     .rate_limit(100, 3600) // 100 uploads per hour
    ///     .build();
    /// ```
    #[must_use]
    pub const fn rate_limit(mut self, max_uploads: usize, window_secs: u64) -> Self {
        self.max_uploads_per_window = Some(max_uploads);
        self.rate_limit_window_secs = Some(window_secs);
        self
    }

    /// Builds the upload policy
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::policy::PolicyBuilder;
    ///
    /// let policy = PolicyBuilder::new()
    ///     .max_file_size(10 * 1024 * 1024)
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> UploadPolicy {
        UploadPolicy {
            max_file_size: self.max_file_size,
            allowed_mime_types: self.allowed_mime_types,
            max_total_storage: self.max_total_storage,
            max_uploads_per_window: self.max_uploads_per_window,
            rate_limit_window_secs: self.rate_limit_window_secs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = UploadPolicy::default();
        assert!(policy.max_file_size().is_some());
        assert!(policy.max_total_storage().is_some());
        assert!(policy.rate_limit().is_some());
    }

    #[test]
    fn test_unrestricted_policy() {
        let policy = UploadPolicy::unrestricted();
        assert!(policy.max_file_size().is_none());
        assert!(policy.allowed_mime_types().is_none());
        assert!(policy.max_total_storage().is_none());
        assert!(policy.rate_limit().is_none());
    }

    #[test]
    fn test_restrictive_policy() {
        let policy = UploadPolicy::restrictive();
        assert_eq!(policy.max_file_size(), Some(1024 * 1024));
        assert!(policy.allowed_mime_types().is_some());
    }

    #[test]
    fn test_allows_upload_success() {
        let policy = PolicyBuilder::new()
            .max_file_size(10 * 1024 * 1024)
            .allowed_mime_types(vec!["image/jpeg"])
            .max_total_storage(100 * 1024 * 1024)
            .build();

        let result = policy.allows_upload(5 * 1024 * 1024, "image/jpeg", 50 * 1024 * 1024);
        assert!(result.is_ok());
    }

    #[test]
    fn test_allows_upload_file_too_large() {
        let policy = PolicyBuilder::new()
            .max_file_size(10 * 1024 * 1024)
            .build();

        let result = policy.allows_upload(20 * 1024 * 1024, "image/jpeg", 0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StorageError::FileSizeExceeded { .. }
        ));
    }

    #[test]
    fn test_allows_upload_invalid_mime() {
        let policy = PolicyBuilder::new()
            .allowed_mime_types(vec!["image/jpeg", "image/png"])
            .build();

        let result = policy.allows_upload(1024, "application/pdf", 0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StorageError::InvalidMimeType { .. }
        ));
    }

    #[test]
    fn test_allows_upload_quota_exceeded() {
        let policy = PolicyBuilder::new()
            .max_total_storage(100 * 1024 * 1024)
            .build();

        let result = policy.allows_upload(10 * 1024 * 1024, "image/jpeg", 95 * 1024 * 1024);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StorageError::QuotaExceeded
        ));
    }

    #[test]
    fn test_policy_builder() {
        let policy = PolicyBuilder::new()
            .max_file_size(5 * 1024 * 1024)
            .allowed_mime_types(vec!["image/jpeg"])
            .max_total_storage(50 * 1024 * 1024)
            .rate_limit(100, 3600)
            .build();

        assert_eq!(policy.max_file_size(), Some(5 * 1024 * 1024));
        assert!(policy.allowed_mime_types().is_some());
        assert_eq!(policy.max_total_storage(), Some(50 * 1024 * 1024));
        assert_eq!(policy.rate_limit(), Some((100, 3600)));
    }

    #[test]
    fn test_rate_limit_getters() {
        let policy = PolicyBuilder::new().rate_limit(50, 1800).build();

        let (max_uploads, window_secs) = policy.rate_limit().unwrap();
        assert_eq!(max_uploads, 50);
        assert_eq!(window_secs, 1800);
    }
}
