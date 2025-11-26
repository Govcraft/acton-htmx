//! MIME type validation with magic number checking
//!
//! This module provides secure MIME type validation that goes beyond trusting
//! the Content-Type header. It uses magic number detection to verify file types
//! based on actual file content.
//!
//! # Security
//!
//! **Never trust client-provided Content-Type headers alone!** Attackers can easily
//! forge headers to bypass simple MIME type checks. This module uses the `infer` crate
//! to examine file signatures (magic numbers) to determine the actual file type.
//!
//! # Examples
//!
//! ```rust
//! use acton_htmx::storage::{UploadedFile, validation::MimeValidator};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let file = UploadedFile::new(
//!     "image.jpg",
//!     "image/jpeg", // Client-provided (could be forged!)
//!     vec![0xFF, 0xD8, 0xFF], // JPEG magic bytes
//! );
//!
//! let validator = MimeValidator::new();
//!
//! // Verify the file is actually a JPEG based on content
//! validator.validate_against_magic(&file, &["image/jpeg"])?;
//!
//! // This would fail even if Content-Type says "image/jpeg"
//! // because the magic bytes don't match
//! let fake = UploadedFile::new(
//!     "fake.jpg",
//!     "image/jpeg", // Lies!
//!     b"not actually a jpeg".to_vec(),
//! );
//! assert!(validator.validate_against_magic(&fake, &["image/jpeg"]).is_err());
//! # Ok(())
//! # }
//! ```

use super::types::{StorageError, StorageResult, UploadedFile};

/// MIME type validator using magic number detection
///
/// This validator uses file signatures (magic numbers) to determine the actual
/// file type, providing security against forged Content-Type headers.
#[derive(Debug, Clone, Default)]
pub struct MimeValidator {
    /// Whether to strictly enforce magic number matches
    strict: bool,
}

impl MimeValidator {
    /// Creates a new MIME validator
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::validation::MimeValidator;
    ///
    /// let validator = MimeValidator::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self { strict: true }
    }

    /// Creates a validator in permissive mode
    ///
    /// In permissive mode, if the magic number cannot be detected,
    /// the validator falls back to checking the Content-Type header.
    /// This is useful for file types without clear magic numbers.
    ///
    /// **Warning**: Permissive mode is less secure. Use only when necessary.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::validation::MimeValidator;
    ///
    /// let validator = MimeValidator::permissive();
    /// ```
    #[must_use]
    pub const fn permissive() -> Self {
        Self { strict: false }
    }

    /// Detects the actual MIME type from file content
    ///
    /// Uses magic number detection to determine the file type.
    /// Returns `None` if the file type cannot be determined.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::{UploadedFile, validation::MimeValidator};
    ///
    /// let file = UploadedFile::new(
    ///     "test.jpg",
    ///     "application/octet-stream",
    ///     vec![0xFF, 0xD8, 0xFF], // JPEG magic bytes
    /// );
    ///
    /// let validator = MimeValidator::new();
    /// let detected = validator.detect_mime(&file);
    /// assert_eq!(detected, Some("image/jpeg"));
    /// ```
    #[must_use]
    pub fn detect_mime(&self, file: &UploadedFile) -> Option<&'static str> {
        infer::get(&file.data).map(|kind| kind.mime_type())
    }

    /// Validates file against allowed MIME types using magic number detection
    ///
    /// This is the most secure validation method as it checks the actual file content
    /// rather than trusting the Content-Type header.
    ///
    /// # Errors
    ///
    /// Returns `StorageError::InvalidMimeType` if:
    /// - The detected type is not in `allowed_types`
    /// - In strict mode: The file type cannot be detected
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::{UploadedFile, validation::MimeValidator};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = UploadedFile::new(
    ///     "photo.png",
    ///     "image/png",
    ///     vec![0x89, 0x50, 0x4E, 0x47], // PNG magic bytes
    /// );
    ///
    /// let validator = MimeValidator::new();
    /// validator.validate_against_magic(&file, &["image/png", "image/jpeg"])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_against_magic(
        &self,
        file: &UploadedFile,
        allowed_types: &[&str],
    ) -> StorageResult<()> {
        match self.detect_mime(file) {
            Some(detected_type) => {
                if !allowed_types.contains(&detected_type) {
                    return Err(StorageError::InvalidMimeType {
                        expected: allowed_types.iter().map(|s| (*s).to_string()).collect(),
                        actual: detected_type.to_string(),
                    });
                }
                Ok(())
            }
            None => {
                if self.strict {
                    // In strict mode, inability to detect is an error
                    Err(StorageError::InvalidMimeType {
                        expected: allowed_types.iter().map(|s| (*s).to_string()).collect(),
                        actual: "unknown (could not detect from content)".to_string(),
                    })
                } else {
                    // In permissive mode, fall back to Content-Type header
                    file.validate_mime(allowed_types)
                }
            }
        }
    }

    /// Validates that the Content-Type header matches the detected type
    ///
    /// This ensures that the client-provided Content-Type header is accurate.
    /// Useful for detecting mismatches that might indicate malicious uploads.
    ///
    /// # Errors
    ///
    /// Returns `StorageError::InvalidMimeType` if the header doesn't match detected type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::{UploadedFile, validation::MimeValidator};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Honest upload - header matches content
    /// let honest = UploadedFile::new(
    ///     "photo.jpg",
    ///     "image/jpeg",
    ///     vec![0xFF, 0xD8, 0xFF], // JPEG magic bytes
    /// );
    ///
    /// let validator = MimeValidator::new();
    /// validator.validate_header_matches_content(&honest)?;
    ///
    /// // Dishonest upload - header lies about content
    /// let dishonest = UploadedFile::new(
    ///     "malware.jpg", // Claims to be JPEG
    ///     "image/jpeg",
    ///     b"#!/bin/sh\nrm -rf /".to_vec(), // But it's a shell script!
    /// );
    ///
    /// assert!(validator.validate_header_matches_content(&dishonest).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_header_matches_content(&self, file: &UploadedFile) -> StorageResult<()> {
        match self.detect_mime(file) {
            Some(detected_type) => {
                if detected_type != file.content_type {
                    return Err(StorageError::InvalidMimeType {
                        expected: vec![file.content_type.clone()],
                        actual: detected_type.to_string(),
                    });
                }
                Ok(())
            }
            None => {
                // If we can't detect, we can't verify
                if self.strict {
                    Err(StorageError::InvalidMimeType {
                        expected: vec![file.content_type.clone()],
                        actual: "unknown (could not detect from content)".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Checks if the file is an image
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::{UploadedFile, validation::MimeValidator};
    ///
    /// let image = UploadedFile::new(
    ///     "photo.png",
    ///     "image/png",
    ///     vec![0x89, 0x50, 0x4E, 0x47], // PNG magic bytes
    /// );
    ///
    /// let validator = MimeValidator::new();
    /// assert!(validator.is_image(&image));
    ///
    /// let text = UploadedFile::new(
    ///     "doc.txt",
    ///     "text/plain",
    ///     b"Hello, world!".to_vec(),
    /// );
    /// assert!(!validator.is_image(&text));
    /// ```
    #[must_use]
    pub fn is_image(&self, file: &UploadedFile) -> bool {
        self.detect_mime(file)
            .is_some_and(|mime| mime.starts_with("image/"))
    }

    /// Checks if the file is a video
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::{UploadedFile, validation::MimeValidator};
    ///
    /// let video = UploadedFile::new(
    ///     "clip.mp4",
    ///     "video/mp4",
    ///     vec![0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70], // MP4 magic
    /// );
    ///
    /// let validator = MimeValidator::new();
    /// assert!(validator.is_video(&video));
    /// ```
    #[must_use]
    pub fn is_video(&self, file: &UploadedFile) -> bool {
        self.detect_mime(file)
            .is_some_and(|mime| mime.starts_with("video/"))
    }

    /// Checks if the file is a document (PDF, Office, etc.)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::{UploadedFile, validation::MimeValidator};
    ///
    /// let pdf = UploadedFile::new(
    ///     "doc.pdf",
    ///     "application/pdf",
    ///     vec![0x25, 0x50, 0x44, 0x46], // PDF magic bytes
    /// );
    ///
    /// let validator = MimeValidator::new();
    /// assert!(validator.is_document(&pdf));
    /// ```
    #[must_use]
    pub fn is_document(&self, file: &UploadedFile) -> bool {
        const DOCUMENT_TYPES: &[&str] = &[
            "application/pdf",
            "application/msword",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "application/vnd.ms-excel",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "application/vnd.ms-powerpoint",
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ];

        self.detect_mime(file)
            .is_some_and(|mime| DOCUMENT_TYPES.contains(&mime))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Common file magic numbers for testing
    const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];
    const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    const GIF_MAGIC: &[u8] = b"GIF89a";
    const PDF_MAGIC: &[u8] = b"%PDF-1.4";
    const ZIP_MAGIC: &[u8] = &[0x50, 0x4B, 0x03, 0x04];

    #[test]
    fn test_detect_jpeg() {
        let file = UploadedFile::new("test.jpg", "image/jpeg", JPEG_MAGIC.to_vec());
        let validator = MimeValidator::new();
        assert_eq!(validator.detect_mime(&file), Some("image/jpeg"));
    }

    #[test]
    fn test_detect_png() {
        let file = UploadedFile::new("test.png", "image/png", PNG_MAGIC.to_vec());
        let validator = MimeValidator::new();
        assert_eq!(validator.detect_mime(&file), Some("image/png"));
    }

    #[test]
    fn test_detect_gif() {
        let file = UploadedFile::new("test.gif", "image/gif", GIF_MAGIC.to_vec());
        let validator = MimeValidator::new();
        assert_eq!(validator.detect_mime(&file), Some("image/gif"));
    }

    #[test]
    fn test_detect_pdf() {
        let file = UploadedFile::new("test.pdf", "application/pdf", PDF_MAGIC.to_vec());
        let validator = MimeValidator::new();
        assert_eq!(validator.detect_mime(&file), Some("application/pdf"));
    }

    #[test]
    fn test_detect_unknown() {
        let file = UploadedFile::new("test.txt", "text/plain", b"hello".to_vec());
        let validator = MimeValidator::new();
        // Text files don't have magic numbers
        assert_eq!(validator.detect_mime(&file), None);
    }

    #[test]
    fn test_validate_against_magic_success() {
        let file = UploadedFile::new("photo.jpg", "image/jpeg", JPEG_MAGIC.to_vec());
        let validator = MimeValidator::new();
        assert!(validator
            .validate_against_magic(&file, &["image/jpeg", "image/png"])
            .is_ok());
    }

    #[test]
    fn test_validate_against_magic_failure() {
        let file = UploadedFile::new("photo.jpg", "image/jpeg", JPEG_MAGIC.to_vec());
        let validator = MimeValidator::new();
        let result = validator.validate_against_magic(&file, &["image/png", "image/gif"]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StorageError::InvalidMimeType { .. }
        ));
    }

    #[test]
    fn test_validate_against_magic_strict_unknown() {
        let file = UploadedFile::new("test.txt", "text/plain", b"hello".to_vec());
        let validator = MimeValidator::new(); // Strict mode
        let result = validator.validate_against_magic(&file, &["text/plain"]);
        assert!(result.is_err()); // Strict mode fails on unknown
    }

    #[test]
    fn test_validate_against_magic_permissive_unknown() {
        let file = UploadedFile::new("test.txt", "text/plain", b"hello".to_vec());
        let validator = MimeValidator::permissive();
        let result = validator.validate_against_magic(&file, &["text/plain"]);
        assert!(result.is_ok()); // Permissive mode falls back to Content-Type
    }

    #[test]
    fn test_header_matches_content_honest() {
        let file = UploadedFile::new("photo.png", "image/png", PNG_MAGIC.to_vec());
        let validator = MimeValidator::new();
        assert!(validator.validate_header_matches_content(&file).is_ok());
    }

    #[test]
    fn test_header_matches_content_dishonest() {
        // Claim it's a JPEG but it's actually a PNG
        let file = UploadedFile::new("fake.jpg", "image/jpeg", PNG_MAGIC.to_vec());
        let validator = MimeValidator::new();
        let result = validator.validate_header_matches_content(&file);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_matches_content_malicious() {
        // Claim it's an image but upload a shell script
        let file = UploadedFile::new(
            "malware.jpg",
            "image/jpeg",
            b"#!/bin/sh\nrm -rf /".to_vec(),
        );
        let validator = MimeValidator::new();
        let result = validator.validate_header_matches_content(&file);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_image() {
        let validator = MimeValidator::new();

        let jpeg = UploadedFile::new("photo.jpg", "image/jpeg", JPEG_MAGIC.to_vec());
        assert!(validator.is_image(&jpeg));

        let png = UploadedFile::new("photo.png", "image/png", PNG_MAGIC.to_vec());
        assert!(validator.is_image(&png));

        let pdf = UploadedFile::new("doc.pdf", "application/pdf", PDF_MAGIC.to_vec());
        assert!(!validator.is_image(&pdf));
    }

    #[test]
    fn test_is_document() {
        let validator = MimeValidator::new();

        let pdf = UploadedFile::new("doc.pdf", "application/pdf", PDF_MAGIC.to_vec());
        assert!(validator.is_document(&pdf));

        let jpeg = UploadedFile::new("photo.jpg", "image/jpeg", JPEG_MAGIC.to_vec());
        assert!(!validator.is_document(&jpeg));
    }

    #[test]
    fn test_forged_extension() {
        // Attacker renames malware.exe to malware.jpg
        let file = UploadedFile::new(
            "malware.jpg",
            "image/jpeg",
            ZIP_MAGIC.to_vec(), // ZIP/EXE magic
        );

        let validator = MimeValidator::new();

        // This should fail because magic number doesn't match claimed type
        assert!(validator
            .validate_against_magic(&file, &["image/jpeg"])
            .is_err());
    }
}
