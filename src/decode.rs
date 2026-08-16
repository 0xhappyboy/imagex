//! Image decoding module
//!
//! Provides unified decode interface with all functions bound to Imagex.
//! Currently supports PNG and PPM formats with pure Rust implementations.
use crate::{ColorSpace, ImageFormat, ImageInfo, Imagex};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
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
    /// Read a PNG image from file
    pub fn read_png<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        use miniz_oxide::inflate::decompress_to_vec_zlib;
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        // PNG signature: 89 50 4E 47 0D 0A 1A 0A
        let mut signature = [0u8; 8];
        reader.read_exact(&mut signature)?;
        if &signature != b"\x89PNG\r\n\x1A\n" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid PNG signature",
            ));
        }
        let mut width = 0u32;
        let mut height = 0u32;
        let mut bit_depth = 0u8;
        let mut color_type = 0u8;
        let mut has_alpha = false;
        let mut compressed_data = Vec::new();
        // Read chunks
        loop {
            let mut len_buf = [0u8; 4];
            if reader.read_exact(&mut len_buf).is_err() {
                break;
            }
            let chunk_len = u32::from_be_bytes(len_buf);
            let mut chunk_type = [0u8; 4];
            reader.read_exact(&mut chunk_type)?;
            let chunk_type_str = std::str::from_utf8(&chunk_type).unwrap_or("????");
            match chunk_type_str {
                "IHDR" => {
                    let mut ihdr = [0u8; 13];
                    reader.read_exact(&mut ihdr)?;
                    width = u32::from_be_bytes([ihdr[0], ihdr[1], ihdr[2], ihdr[3]]);
                    height = u32::from_be_bytes([ihdr[4], ihdr[5], ihdr[6], ihdr[7]]);
                    bit_depth = ihdr[8];
                    color_type = ihdr[9];
                    has_alpha = color_type == 4 || color_type == 6;
                }
                "IDAT" => {
                    let mut data = vec![0u8; chunk_len as usize];
                    reader.read_exact(&mut data)?;
                    compressed_data.extend(data);
                }
                "IEND" => {
                    break;
                }
                _ => {
                    let mut skip = vec![0u8; chunk_len as usize];
                    reader.read_exact(&mut skip)?;
                }
            }
            let mut crc = [0u8; 4];
            reader.read_exact(&mut crc)?;
        }
        if compressed_data.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "PNG has no IDAT chunks",
            ));
        }
        // Decompress zlib data using miniz_oxide
        let decompressed = decompress_to_vec_zlib(&compressed_data).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("zlib decompression failed: {:?}", e),
            )
        })?;
        // PNG filter decoding
        let bytes_per_pixel = if has_alpha { 4 } else { 3 };
        let stride = (width as usize) * bytes_per_pixel;
        let expected_size = (height as usize) * (stride + 1); // +1 for filter byte per row
        if decompressed.len() < expected_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Decompressed data too small: got {}, expected {}",
                    decompressed.len(),
                    expected_size
                ),
            ));
        }
        // Apply PNG filters (Paeth filter decoding)
        let mut raw_data = Vec::with_capacity((height as usize) * stride);
        let mut prev_row = vec![0u8; stride];
        for row in 0..height as usize {
            let filter_byte = decompressed[row * (stride + 1)];
            let start = row * (stride + 1) + 1;
            let current_row = &decompressed[start..start + stride];
            let mut decoded_row = vec![0u8; stride];
            match filter_byte {
                0 => {
                    // None: no filtering
                    decoded_row.copy_from_slice(current_row);
                }
                1 => {
                    // Sub: subtract left pixel
                    for i in 0..stride {
                        let left = if i >= bytes_per_pixel {
                            decoded_row[i - bytes_per_pixel]
                        } else {
                            0
                        };
                        decoded_row[i] = current_row[i].wrapping_add(left);
                    }
                }
                2 => {
                    // Up: subtract pixel from previous row
                    for i in 0..stride {
                        decoded_row[i] = current_row[i].wrapping_add(prev_row[i]);
                    }
                }
                3 => {
                    // Average: (left + up) / 2
                    for i in 0..stride {
                        let left = if i >= bytes_per_pixel {
                            decoded_row[i - bytes_per_pixel]
                        } else {
                            0
                        };
                        let up = prev_row[i];
                        decoded_row[i] =
                            current_row[i].wrapping_add(((left as u16 + up as u16) / 2) as u8);
                    }
                }
                4 => {
                    // Paeth: predictor function
                    for i in 0..stride {
                        let left = if i >= bytes_per_pixel {
                            decoded_row[i - bytes_per_pixel]
                        } else {
                            0
                        };
                        let up = prev_row[i];
                        let upper_left = if i >= bytes_per_pixel {
                            prev_row[i - bytes_per_pixel]
                        } else {
                            0
                        };
                        let paeth = Self::paeth_predictor(left, up, upper_left);
                        decoded_row[i] = current_row[i].wrapping_add(paeth);
                    }
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        format!("PNG filter type {} not supported", filter_byte),
                    ));
                }
            }
            raw_data.extend_from_slice(&decoded_row);
            prev_row.copy_from_slice(&decoded_row);
        }
        // Convert to RGBA if needed
        let pixel_data = if has_alpha {
            raw_data
        } else {
            // Convert RGB to RGBA (add alpha = 255)
            let mut rgba = Vec::with_capacity(raw_data.len() / 3 * 4);
            for chunk in raw_data.chunks_exact(3) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(chunk[2]);
                rgba.push(255);
            }
            rgba
        };
        let color_space = if has_alpha {
            ColorSpace::Rgba
        } else {
            ColorSpace::Rgb
        };
        let info = ImageInfo {
            width,
            height,
            format: ImageFormat::Png,
            color_space,
            bit_depth,
            has_alpha,
            compression: Some("DEFLATE".to_string()),
        };
        let mut img = Self::from_raw(pixel_data, width, height);
        img.info = Some(info);
        Ok(img)
    }
    /// Paeth predictor function for PNG filter type 4
    fn paeth_predictor(left: u8, up: u8, upper_left: u8) -> u8 {
        let p = left as i16 + up as i16 - upper_left as i16;
        let pa = (p - left as i16).abs();
        let pb = (p - up as i16).abs();
        let pc = (p - upper_left as i16).abs();
        if pa <= pb && pa <= pc {
            left
        } else if pb <= pc {
            up
        } else {
            upper_left
        }
    }
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
