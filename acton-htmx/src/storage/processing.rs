//! Image processing utilities for file uploads
//!
//! This module provides utilities for processing uploaded images:
//! - Thumbnail generation
//! - Image resizing
//! - Format conversion
//! - EXIF metadata stripping (for privacy)
//!
//! # Examples
//!
//! ```rust,no_run
//! use acton_htmx::storage::{UploadedFile, processing::ImageProcessor};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let file = UploadedFile::new(
//!     "photo.jpg",
//!     "image/jpeg",
//!     vec![/* ... */],
//! );
//!
//! let processor = ImageProcessor::new();
//!
//! // Generate thumbnail
//! let thumbnail = processor.generate_thumbnail(&file, 200, 200)?;
//!
//! // Resize image
//! let resized = processor.resize(&file, 800, 600)?;
//!
//! // Strip EXIF metadata
//! let stripped = processor.strip_exif(&file)?;
//! # Ok(())
//! # }
//! ```

use super::types::{StorageError, StorageResult, UploadedFile};
use image::{
    imageops::FilterType, DynamicImage, ImageFormat, ImageReader,
};
use std::io::Cursor;

/// Image processing utilities
///
/// Provides methods for common image operations like resizing,
/// thumbnail generation, and EXIF stripping.
#[derive(Debug, Clone)]
pub struct ImageProcessor {
    /// Default filter for resizing operations
    filter: FilterType,
}

impl Default for ImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageProcessor {
    /// Creates a new image processor with default settings
    ///
    /// Uses `FilterType::Lanczos3` for high-quality resizing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::processing::ImageProcessor;
    ///
    /// let processor = ImageProcessor::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            filter: FilterType::Lanczos3,
        }
    }

    /// Creates a processor with a specific resize filter
    ///
    /// # Examples
    ///
    /// ```rust
    /// use acton_htmx::storage::processing::ImageProcessor;
    /// use image::imageops::FilterType;
    ///
    /// let processor = ImageProcessor::with_filter(FilterType::Nearest);
    /// ```
    #[must_use]
    pub const fn with_filter(filter: FilterType) -> Self {
        Self { filter }
    }

    /// Loads an image from an uploaded file
    ///
    /// # Errors
    ///
    /// Returns error if the file is not a valid image
    fn load_image(file: &UploadedFile) -> StorageResult<DynamicImage> {
        let reader = ImageReader::new(Cursor::new(&file.data))
            .with_guessed_format()
            .map_err(|e| StorageError::Other(format!("Failed to read image: {e}")))?;

        reader
            .decode()
            .map_err(|e| StorageError::Other(format!("Failed to decode image: {e}")))
    }

    /// Detects the image format from the file data
    fn detect_format(file: &UploadedFile) -> StorageResult<ImageFormat> {
        ImageFormat::from_mime_type(&file.content_type)
            .ok_or_else(|| StorageError::Other(format!("Unsupported image format: {}", file.content_type)))
    }

    /// Encodes an image to bytes
    fn encode_image(
        image: &DynamicImage,
        format: ImageFormat,
    ) -> StorageResult<Vec<u8>> {
        let mut buffer = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut buffer), format)
            .map_err(|e| StorageError::Other(format!("Failed to encode image: {e}")))?;
        Ok(buffer)
    }

    /// Generates a thumbnail from an uploaded image
    ///
    /// Creates a thumbnail that fits within the specified dimensions while
    /// maintaining aspect ratio.
    ///
    /// # Arguments
    ///
    /// * `file` - The uploaded image file
    /// * `max_width` - Maximum width in pixels
    /// * `max_height` - Maximum height in pixels
    ///
    /// # Errors
    ///
    /// Returns error if the file is not a valid image or processing fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use acton_htmx::storage::{UploadedFile, processing::ImageProcessor};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = UploadedFile::new("photo.jpg", "image/jpeg", vec![/* ... */]);
    /// let processor = ImageProcessor::new();
    ///
    /// // Generate 200x200 thumbnail
    /// let thumbnail = processor.generate_thumbnail(&file, 200, 200)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_thumbnail(
        &self,
        file: &UploadedFile,
        max_width: u32,
        max_height: u32,
    ) -> StorageResult<UploadedFile> {
        let img = Self::load_image(file)?;
        let format = Self::detect_format(file)?;

        // Generate thumbnail maintaining aspect ratio
        let thumbnail = img.thumbnail(max_width, max_height);

        let data = Self::encode_image(&thumbnail, format)?;

        Ok(UploadedFile {
            filename: format!("thumb_{}", file.filename),
            content_type: file.content_type.clone(),
            data,
        })
    }

    /// Resizes an image to exact dimensions
    ///
    /// Resizes the image to the specified width and height. This may change
    /// the aspect ratio if the new dimensions don't match the original.
    ///
    /// # Arguments
    ///
    /// * `file` - The uploaded image file
    /// * `width` - Target width in pixels
    /// * `height` - Target height in pixels
    ///
    /// # Errors
    ///
    /// Returns error if the file is not a valid image or processing fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use acton_htmx::storage::{UploadedFile, processing::ImageProcessor};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = UploadedFile::new("photo.jpg", "image/jpeg", vec![/* ... */]);
    /// let processor = ImageProcessor::new();
    ///
    /// // Resize to exactly 800x600
    /// let resized = processor.resize(&file, 800, 600)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn resize(
        &self,
        file: &UploadedFile,
        width: u32,
        height: u32,
    ) -> StorageResult<UploadedFile> {
        let img = Self::load_image(file)?;
        let format = Self::detect_format(file)?;

        let resized = img.resize_exact(width, height, self.filter);

        let data = Self::encode_image(&resized, format)?;

        Ok(UploadedFile {
            filename: format!("{}x{}_{}", width, height, file.filename),
            content_type: file.content_type.clone(),
            data,
        })
    }

    /// Converts an image to a different format
    ///
    /// # Arguments
    ///
    /// * `file` - The uploaded image file
    /// * `target_format` - The desired output format (e.g., "image/png")
    ///
    /// # Errors
    ///
    /// Returns error if the file is not a valid image or format is unsupported
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use acton_htmx::storage::{UploadedFile, processing::ImageProcessor};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = UploadedFile::new("photo.jpg", "image/jpeg", vec![/* ... */]);
    /// let processor = ImageProcessor::new();
    ///
    /// // Convert JPEG to PNG
    /// let png = processor.convert_format(&file, "image/png")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn convert_format(
        &self,
        file: &UploadedFile,
        target_format: &str,
    ) -> StorageResult<UploadedFile> {
        let img = Self::load_image(file)?;

        let format = ImageFormat::from_mime_type(target_format)
            .ok_or_else(|| StorageError::Other(format!("Unsupported target format: {target_format}")))?;

        let data = Self::encode_image(&img, format)?;

        // Update filename extension
        let new_filename = file.extension().map_or_else(
            || format!("{}.{}", file.filename, format_extension(format)),
            |ext| file.filename.replace(&format!(".{ext}"), &format!(".{}", format_extension(format))),
        );

        Ok(UploadedFile {
            filename: new_filename,
            content_type: target_format.to_string(),
            data,
        })
    }

    /// Strips EXIF metadata from an image for privacy
    ///
    /// Removes all EXIF metadata (location, camera info, etc.) from an image.
    /// This is important for user privacy as EXIF data can contain sensitive
    /// information like GPS coordinates.
    ///
    /// # Errors
    ///
    /// Returns error if the file is not a valid image or processing fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use acton_htmx::storage::{UploadedFile, processing::ImageProcessor};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = UploadedFile::new("photo.jpg", "image/jpeg", vec![/* ... */]);
    /// let processor = ImageProcessor::new();
    ///
    /// // Remove all EXIF metadata
    /// let stripped = processor.strip_exif(&file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn strip_exif(&self, file: &UploadedFile) -> StorageResult<UploadedFile> {
        let img = Self::load_image(file)?;
        let format = Self::detect_format(file)?;

        // Re-encoding the image without EXIF data effectively strips it
        let data = Self::encode_image(&img, format)?;

        Ok(UploadedFile {
            filename: file.filename.clone(),
            content_type: file.content_type.clone(),
            data,
        })
    }

    /// Gets image dimensions without fully decoding
    ///
    /// This is faster than loading the full image when you only need dimensions.
    ///
    /// # Errors
    ///
    /// Returns error if the file is not a valid image
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use acton_htmx::storage::{UploadedFile, processing::ImageProcessor};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = UploadedFile::new("photo.jpg", "image/jpeg", vec![/* ... */]);
    /// let processor = ImageProcessor::new();
    ///
    /// let (width, height) = processor.get_dimensions(&file)?;
    /// println!("Image is {width}x{height}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_dimensions(&self, file: &UploadedFile) -> StorageResult<(u32, u32)> {
        let reader = ImageReader::new(Cursor::new(&file.data))
            .with_guessed_format()
            .map_err(|e| StorageError::Other(format!("Failed to read image: {e}")))?;

        reader
            .into_dimensions()
            .map_err(|e| StorageError::Other(format!("Failed to get dimensions: {e}")))
    }
}

/// Helper function to get file extension for image format
const fn format_extension(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Gif => "gif",
        ImageFormat::WebP => "webp",
        ImageFormat::Tiff => "tiff",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Ico => "ico",
        ImageFormat::Avif => "avif",
        _ => "bin",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    /// Helper to create a test PNG image
    fn create_test_png(width: u32, height: u32) -> Vec<u8> {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |_, _| {
            Rgb([255, 0, 0]) // Red pixel
        });

        let mut buffer = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
            .unwrap();
        buffer
    }

    #[test]
    fn test_get_dimensions() {
        let png_data = create_test_png(10, 20);
        let file = UploadedFile::new("test.png", "image/png", png_data);
        let processor = ImageProcessor::new();

        let (width, height) = processor.get_dimensions(&file).unwrap();
        assert_eq!(width, 10);
        assert_eq!(height, 20);
    }

    #[test]
    fn test_load_image() {
        let png_data = create_test_png(5, 5);
        let file = UploadedFile::new("test.png", "image/png", png_data);

        let img = ImageProcessor::load_image(&file).unwrap();
        assert_eq!(img.width(), 5);
        assert_eq!(img.height(), 5);
    }

    #[test]
    fn test_strip_exif() {
        let png_data = create_test_png(10, 10);
        let file = UploadedFile::new("test.png", "image/png", png_data);
        let processor = ImageProcessor::new();

        let stripped = processor.strip_exif(&file).unwrap();
        assert_eq!(stripped.content_type, "image/png");
        assert!(!stripped.data.is_empty());
    }

    #[test]
    fn test_resize() {
        let png_data = create_test_png(20, 30);
        let file = UploadedFile::new("test.png", "image/png", png_data);
        let processor = ImageProcessor::new();

        let resized = processor.resize(&file, 10, 15).unwrap();
        assert_eq!(resized.content_type, "image/png");

        // Verify new dimensions
        let (width, height) = processor.get_dimensions(&resized).unwrap();
        assert_eq!(width, 10);
        assert_eq!(height, 15);
    }

    #[test]
    fn test_thumbnail() {
        let png_data = create_test_png(100, 100);
        let file = UploadedFile::new("test.png", "image/png", png_data);
        let processor = ImageProcessor::new();

        let thumb = processor.generate_thumbnail(&file, 50, 50).unwrap();
        assert_eq!(thumb.content_type, "image/png");
        assert!(thumb.filename.starts_with("thumb_"));

        // Verify thumbnail is smaller
        let (width, height) = processor.get_dimensions(&thumb).unwrap();
        assert!(width <= 50);
        assert!(height <= 50);
    }

    #[test]
    fn test_invalid_image() {
        let file = UploadedFile::new("test.png", "image/png", b"not an image".to_vec());
        let processor = ImageProcessor::new();

        assert!(processor.get_dimensions(&file).is_err());
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(format_extension(ImageFormat::Png), "png");
        assert_eq!(format_extension(ImageFormat::Jpeg), "jpg");
        assert_eq!(format_extension(ImageFormat::Gif), "gif");
        assert_eq!(format_extension(ImageFormat::WebP), "webp");
    }
}
