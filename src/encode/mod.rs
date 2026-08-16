//! Image encoding modules
//!
//! Each image format has its own module for better maintainability.
pub mod png;
pub mod ppm;
use crate::{ImageFormat, Imagex};
use std::path::Path;
impl Imagex {
    /// Write image to file in specified format
    ///
    /// # Example
    /// ```
    /// use imagex::{Imagex, ImageFormat};
    ///
    /// let img = Imagex::new(1920, 1080);
    /// img.write("output.png", ImageFormat::Png).unwrap();
    /// ```
    pub fn write<P: AsRef<Path>>(&self, path: P, format: ImageFormat) -> std::io::Result<()> {
        match format {
            ImageFormat::Png => self.write_png(path),
            ImageFormat::Ppm => self.write_ppm(path),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("Export format not yet supported: {:?}", format),
            )),
        }
    }
}
/// CRC32 calculation (for PNG chunk validation)
pub(crate) fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFFFFFF
}
