//! Image decoding modules
//!
//! Each image format has its own module for better maintainability.
pub mod png;
pub mod ppm;
use crate::{ImageFormat, ImageInfo, Imagex};
use std::path::Path;
impl Imagex {
    /// Read an image from file, auto-detecting format
    ///
    /// # Example
    /// ```
    /// use imagex::Imagex;
    ///
    /// let img = Imagex::read("photo.png").unwrap();
    /// println!("Image info: {:?}", img.info());
    /// ```
    pub fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref();
        let format = ImageFormat::from_path(path.to_str().unwrap_or(""));
        match format {
            ImageFormat::Png => Self::read_png(path),
            ImageFormat::Ppm => Self::read_ppm(path),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unsupported image format: {}", path.display()),
            )),
        }
    }
    /// Get image metadata
    pub fn info(&self) -> Option<&ImageInfo> {
        self.info.as_ref()
    }
    /// Get image format
    pub fn format(&self) -> Option<ImageFormat> {
        self.info.as_ref().map(|info| info.format)
    }
    /// Check if image has alpha channel
    pub fn has_alpha(&self) -> bool {
        self.info.as_ref().map_or(false, |info| info.has_alpha)
    }
}
