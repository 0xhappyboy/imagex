use crate::{ColorSpace, ImageFormat, ImageInfo, Imagex};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
impl Imagex {
    /// Read a PPM image from file (P6 binary format)
    pub fn read_ppm<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        // Read magic number
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let magic = line.trim();
        if magic != "P6" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Only P6 format is supported, got: {}", magic),
            ));
        }
        // Skip comments
        let mut line = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line)?;
            if !line.starts_with('#') {
                break;
            }
        }
        // Read dimensions
        let dims: Vec<&str> = line.trim().split_whitespace().collect();
        if dims.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid PPM header: missing width/height",
            ));
        }
        let width: u32 = dims[0]
            .parse()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid width"))?;
        let height: u32 = dims[1]
            .parse()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid height"))?;
        // Read max value
        let mut max_val = String::new();
        reader.read_line(&mut max_val)?;
        let _max: u32 = max_val.trim().parse().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid max color value")
        })?;
        // Read pixel data
        let pixel_count = (width * height) as usize;
        let mut rgb_data = vec![0u8; pixel_count * 3];
        reader.read_exact(&mut rgb_data)?;
        let info = ImageInfo {
            width,
            height,
            format: ImageFormat::Ppm,
            color_space: ColorSpace::Rgb,
            bit_depth: 8,
            has_alpha: false,
            compression: None,
        };
        let mut img = Self::from_rgb8(&rgb_data, width, height);
        img.info = Some(info);
        Ok(img)
    }
}
