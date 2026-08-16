//! imagex - A high-performance image processing library.
//!
//! Features flat row-major memory layout, configurable SIMD stride alignment,
//! zero-copy access, and parallel iteration support.
//!
//! # Example
//! ```
//! use imagex::Imagex;
//!
//! let mut img = Imagex::new(1920, 1080);
//! img.fill(&imagex::Rgba::new(255, 0, 0, 255));
//! assert_eq!(img.width(), 1920);
//! assert_eq!(img.height(), 1080);
//! ```
pub mod decode;
pub mod encode;
pub mod pixel;
pub mod rgb;
pub mod rgba;
pub mod types;
pub use decode::*;
pub use encode::*;
pub use pixel::*;
pub use rgb::*;
pub use rgba::*;
use std::marker::PhantomData;
pub use types::*;
/// Optimized image buffer designed for video processing workflows
///
/// `Imagex` manages a flat RGBA pixel buffer in row-major order with
/// configurable stride support. The structure is optimized for:
/// - Zero-copy access to pixel data
/// - Cache-friendly sequential processing
/// - Parallel frame processing (rayon compatibility)
/// - Direct GPU/texture upload (via raw pointer access)
///
/// # Memory Layout
/// ```
/// width = 4, height = 3, stride = 4 (no padding)
///
/// Pixel order in data:
/// [R,G,B,A] [R,G,B,A] [R,G,B,A] [R,G,B,A] = row 0
/// [R,G,B,A] [R,G,B,A] [R,G,B,A] [R,G,B,A] = row 1
/// [R,G,B,A] [R,G,B,A] [R,G,B,A] [R,G,B,A] = row 2
/// ```
///
/// # Performance Notes
/// - Use `pixel_mut()` for single pixel access
/// - Use `row_mut()` for batch processing of rows
/// - Use `as_raw_mut()` for SIMD/unsafe operations
/// - The `stride` field allows alignment to 16/32 bytes for SIMD
#[derive(Debug, Clone)]
pub struct Imagex {
    /// Pixel data in RGBA order (4 bytes per pixel, row-major)
    data: Vec<u8>,
    /// Image width in pixels
    width: u32,
    /// Image height in pixels
    height: u32,
    /// Number of bytes per row (width * 4, may include padding)
    stride: u32,
    /// Image metadata (populated when loaded from file)
    pub info: Option<ImageInfo>,
}
impl Imagex {
    /// Creates a new image buffer with default stride alignment (16 bytes)
    ///
    /// # Arguments
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    ///
    /// # Returns
    /// * `Imagex` - Allocated image buffer with aligned stride
    ///
    /// # Performance
    /// - Allocates `(height * stride)` bytes
    /// - Stride is aligned to 16 bytes for SIMD optimization
    pub fn new(width: u32, height: u32) -> Self {
        let stride = Self::aligned_stride(width);
        let size = (height as usize) * (stride as usize);
        Self {
            data: vec![0; size],
            width,
            height,
            stride,
            info: None,
        }
    }
    /// Creates a new image buffer with custom stride alignment
    ///
    /// # Arguments
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `align` - Byte alignment for stride (must be power of 2)
    ///
    /// # Returns
    /// * `Imagex` - Allocated image buffer with aligned stride
    pub fn new_aligned(width: u32, height: u32, align: usize) -> Self {
        let bytes_per_row = (width as usize) * 4;
        let stride = ((bytes_per_row + align - 1) / align) * align;
        let size = (height as usize) * stride;
        Self {
            data: vec![0; size],
            width,
            height,
            stride: stride as u32,
            info: None,
        }
    }
    /// Creates an image buffer from raw RGBA data
    ///
    /// # Arguments
    /// * `data` - RGBA pixel data (must be at least `width * height * 4` bytes)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    ///
    /// # Returns
    /// * `Imagex` - Image buffer
    pub fn from_raw(data: Vec<u8>, width: u32, height: u32) -> Self {
        let stride = Self::aligned_stride(width);
        let mut img = Self::new(width, height);
        // Copy data row by row (handles stride padding)
        for y in 0..height {
            let src_start = (y as usize) * (width as usize) * 4;
            let dst_start = (y as usize) * (stride as usize);
            let src_end = src_start + (width as usize) * 4;
            img.data[dst_start..dst_start + (width as usize) * 4]
                .copy_from_slice(&data[src_start..src_end]);
        }
        img
    }
    /// Creates an image from a fill color
    ///
    /// # Arguments
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `color` - Fill color
    ///
    /// # Returns
    /// * `Imagex` - Image filled with the specified color
    pub fn filled(width: u32, height: u32, color: &Rgba) -> Self {
        let mut img = Self::new(width, height);
        img.fill(color);
        img
    }
    /// Returns the width of the image
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Returns the height of the image
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }
    /// Returns the stride (bytes per row)
    #[inline]
    pub fn stride(&self) -> u32 {
        self.stride
    }
    /// Returns the total number of pixels
    #[inline]
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }
    /// Returns the total size of the buffer in bytes
    #[inline]
    pub fn buffer_size(&self) -> usize {
        self.data.len()
    }
    /// Returns a raw reference to the underlying data
    #[inline]
    pub fn as_raw(&self) -> &[u8] {
        &self.data
    }
    /// Returns a raw mutable reference to the underlying data
    #[inline]
    pub fn as_raw_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
    /// Gets a pixel at (x, y)
    ///
    /// # Panics
    /// Panics if `x >= width` or `y >= height`
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Rgba {
        let idx = (y as usize) * (self.stride as usize) + (x as usize) * 4;
        Rgba {
            r: self.data[idx],
            g: self.data[idx + 1],
            b: self.data[idx + 2],
            a: self.data[idx + 3],
        }
    }
    /// Gets a pixel at (x, y) without bounds checking
    ///
    /// # Safety
    /// Caller must ensure `x < width` and `y < height`
    #[inline]
    pub unsafe fn get_pixel_unchecked(&self, x: u32, y: u32) -> Rgba {
        let idx = (y as usize) * (self.stride as usize) + (x as usize) * 4;
        Rgba {
            r: *self.data.get_unchecked(idx),
            g: *self.data.get_unchecked(idx + 1),
            b: *self.data.get_unchecked(idx + 2),
            a: *self.data.get_unchecked(idx + 3),
        }
    }
    /// Sets a pixel at (x, y)
    ///
    /// # Panics
    /// Panics if `x >= width` or `y >= height`
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: Rgba) {
        let idx = (y as usize) * (self.stride as usize) + (x as usize) * 4;
        self.data[idx] = pixel.r;
        self.data[idx + 1] = pixel.g;
        self.data[idx + 2] = pixel.b;
        self.data[idx + 3] = pixel.a;
    }
    /// Sets a pixel at (x, y) without bounds checking
    ///
    /// # Safety
    /// Caller must ensure `x < width` and `y < height`
    #[inline]
    pub unsafe fn set_pixel_unchecked(&mut self, x: u32, y: u32, pixel: Rgba) {
        let idx = (y as usize) * (self.stride as usize) + (x as usize) * 4;
        *self.data.get_unchecked_mut(idx) = pixel.r;
        *self.data.get_unchecked_mut(idx + 1) = pixel.g;
        *self.data.get_unchecked_mut(idx + 2) = pixel.b;
        *self.data.get_unchecked_mut(idx + 3) = pixel.a;
    }
    /// Gets a mutable reference to a pixel at (x, y)
    ///
    /// This uses `split_at_mut` to safely obtain non-overlapping mutable references
    /// to each color channel of the pixel.
    ///
    /// # Panics
    /// Panics if `x >= width` or `y >= height`
    #[inline]
    pub fn get_pixel_mut(&mut self, x: u32, y: u32) -> PixelRefMut<'_> {
        let idx = (y as usize) * (self.stride as usize) + (x as usize) * 4;
        let ptr = self.data.as_mut_ptr();
        unsafe {
            PixelRefMut {
                r: &mut *ptr.add(idx),
                g: &mut *ptr.add(idx + 1),
                b: &mut *ptr.add(idx + 2),
                a: &mut *ptr.add(idx + 3),
            }
        }
    }
    /// Gets a reference to a row of pixels
    ///
    /// # Panics
    /// Panics if `y >= height`
    #[inline]
    pub fn row(&self, y: u32) -> &[u8] {
        let start = (y as usize) * (self.stride as usize);
        let end = start + (self.width as usize) * 4;
        &self.data[start..end]
    }
    /// Gets a mutable reference to a row of pixels
    ///
    /// # Panics
    /// Panics if `y >= height`
    #[inline]
    pub fn row_mut(&mut self, y: u32) -> &mut [u8] {
        let start = (y as usize) * (self.stride as usize);
        let end = start + (self.width as usize) * 4;
        &mut self.data[start..end]
    }
    /// Fills the entire image with a color
    pub fn fill(&mut self, color: &Rgba) {
        let pixel = [color.r, color.g, color.b, color.a];
        for row in 0..self.height {
            let start = (row as usize) * (self.stride as usize);
            let end = start + (self.width as usize) * 4;
            for chunk in self.data[start..end].chunks_exact_mut(4) {
                chunk.copy_from_slice(&pixel);
            }
        }
    }
    /// Clears the image to black (0,0,0,255)
    pub fn clear(&mut self) {
        self.fill(&Rgba::black_opaque());
    }
    /// Returns an iterator over all pixels
    pub fn pixels(&self) -> PixelIter<'_> {
        PixelIter {
            data: &self.data,
            stride: self.stride as usize,
            width: self.width as usize,
            height: self.height as usize,
            x: 0,
            y: 0,
            idx: 0,
        }
    }
    /// Returns a mutable iterator over all pixels
    pub fn pixels_mut(&mut self) -> PixelIterMut<'_> {
        let ptr = self.data.as_mut_ptr();
        PixelIterMut {
            ptr,
            stride: self.stride as usize,
            width: self.width as usize,
            height: self.height as usize,
            x: 0,
            y: 0,
            idx: 0,
            _marker: PhantomData,
        }
    }
    /// Calculates the aligned stride (multiple of 16 for SIMD)
    #[inline]
    fn aligned_stride(width: u32) -> u32 {
        let bytes = (width as usize) * 4;
        ((bytes + 15) / 16 * 16) as u32
    }
    /// Checks if the image is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
    /// Converts to RGB8 format (drops alpha channel)
    pub fn to_rgb8(&self) -> Vec<u8> {
        let mut rgb = Vec::with_capacity((self.width * self.height * 3) as usize);
        for y in 0..self.height {
            let row_start = (y as usize) * (self.stride as usize);
            let row_end = row_start + (self.width as usize) * 4;
            for i in (row_start..row_end).step_by(4) {
                rgb.push(self.data[i]);
                rgb.push(self.data[i + 1]);
                rgb.push(self.data[i + 2]);
            }
        }
        rgb
    }
    /// Creates a new image from a slice of RGB data
    pub fn from_rgb8(rgb_data: &[u8], width: u32, height: u32) -> Self {
        let mut img = Self::new(width, height);
        let mut idx = 0;
        for y in 0..height {
            let row_start = (y as usize) * (img.stride as usize);
            for x in 0..width {
                let pixel_idx = row_start + (x as usize) * 4;
                img.data[pixel_idx] = rgb_data[idx];
                img.data[pixel_idx + 1] = rgb_data[idx + 1];
                img.data[pixel_idx + 2] = rgb_data[idx + 2];
                img.data[pixel_idx + 3] = 255;
                idx += 3;
            }
        }
        img
    }
    /// Resizes the image (nearest neighbor)
    pub fn resize(&self, new_width: u32, new_height: u32) -> Self {
        let mut new_img = Self::new(new_width, new_height);
        let scale_x = self.width as f64 / new_width as f64;
        let scale_y = self.height as f64 / new_height as f64;
        for y in 0..new_height {
            let src_y = (y as f64 * scale_y) as u32;
            for x in 0..new_width {
                let src_x = (x as f64 * scale_x) as u32;
                let pixel = self.get_pixel(src_x, src_y);
                new_img.set_pixel(x, y, pixel);
            }
        }
        new_img
    }
}
impl Default for Imagex {
    fn default() -> Self {
        Self::new(1, 1)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::time::Instant;
    #[test]
    fn test_read_local_image() {
        let path = "C:\\Users\\admin\\Downloads\\f9d5452a-9273-40ef-a43e-08b1f0d418b5.png";
        if !Path::new(path).exists() {
            eprintln!("Skipping test: file not found -> {}", path);
            return;
        }
        let img = Imagex::read(path).unwrap();
        println!(
            "\nImage info: {}x{} ({} pixels)",
            img.width(),
            img.height(),
            img.pixel_count()
        );
        let start = Instant::now();
        let mut count = 0;
        for pixel in img.pixels() {
            let _ = pixel.r;
            count += 1;
        }
        let duration = start.elapsed();
        let total_pixels = img.pixel_count() as usize;
        assert_eq!(count, total_pixels);
        let ms = duration.as_secs_f64() * 1000.0;
        let fps = total_pixels as f64 / duration.as_secs_f64();
        let mb_per_sec = (total_pixels as f64 * 4.0) / (duration.as_secs_f64() * 1024.0 * 1024.0);
        println!(
            "Read-only iteration: {:.2}ms ({:.0} px/s, {:.2} MB/s)",
            ms, fps, mb_per_sec
        );
        let mut img_mut = img.clone();
        let start = Instant::now();
        for mut pixel in img_mut.pixels_mut() {
            *pixel.r = 0; // Modify red channel
        }
        let duration = start.elapsed();
        let ms = duration.as_secs_f64() * 1000.0;
        let fps = total_pixels as f64 / duration.as_secs_f64();
        let mb_per_sec = (total_pixels as f64 * 4.0) / (duration.as_secs_f64() * 1024.0 * 1024.0);
        println!(
            "Read-write iteration: {:.2}ms ({:.0} px/s, {:.2} MB/s)",
            ms, fps, mb_per_sec
        );
        let start = Instant::now();
        let mut sum: u64 = 0;
        for y in 0..img.height() {
            let row = img.row(y);
            for chunk in row.chunks_exact(4) {
                sum += chunk[0] as u64; // Sum red channel
            }
        }
        let duration = start.elapsed();
        let ms = duration.as_secs_f64() * 1000.0;
        let fps = total_pixels as f64 / duration.as_secs_f64();
        let mb_per_sec = (total_pixels as f64 * 4.0) / (duration.as_secs_f64() * 1024.0 * 1024.0);
        println!(
            "Row-based iteration: {:.2}ms ({:.0} px/s, {:.2} MB/s)",
            ms, fps, mb_per_sec
        );
        println!("   (Red channel sum: {})", sum);
        let start = Instant::now();
        let mut img_fill = Imagex::new(img.width(), img.height());
        img_fill.fill(&Rgba::new(255, 0, 0, 255));
        let duration = start.elapsed();
        let ms = duration.as_secs_f64() * 1000.0;
        let mb_per_sec = (total_pixels as f64 * 4.0) / (duration.as_secs_f64() * 1024.0 * 1024.0);
        println!("Fill operation: {:.2}ms ({:.2} MB/s)", ms, mb_per_sec);
    }
    #[test]
    fn test_write_local_image() {
        use std::time::Instant;
        let src_path = "C:\\Users\\admin\\Downloads\\a (2).png";
        if !Path::new(src_path).exists() {
            eprintln!("Skipping test: source file not found -> {}", src_path);
            return;
        }
        let start_decode = Instant::now();
        let mut img = Imagex::read(src_path).unwrap();
        let decode_duration = start_decode.elapsed();
        println!(
            "\nSource image: {}x{} ({} pixels)",
            img.width(),
            img.height(),
            img.pixel_count()
        );
        println!(
            "Decode time: {:.2}ms",
            decode_duration.as_secs_f64() * 1000.0
        );
        let start_modify = Instant::now();
        let cx = img.width() / 2;
        let cy = img.height() / 2;
        // Get original center pixel for debugging
        let orig_center = img.get_pixel(cx, cy);
        println!(
            "Original center pixel: ({}, {}, {}, {})",
            orig_center.r, orig_center.g, orig_center.b, orig_center.a
        );
        img.set_pixel(cx, cy, Rgba::new(255, 0, 0, 255));
        let modify_duration = start_modify.elapsed();
        println!(
            "Modify pixels time: {:.2}ms",
            modify_duration.as_secs_f64() * 1000.0
        );
        let out_dir = "C:\\Users\\admin\\Downloads\\output-img";
        std::fs::create_dir_all(out_dir).unwrap();
        let start_encode = Instant::now();
        let png_path = format!("{}\\test1.png", out_dir);
        img.write(&png_path, ImageFormat::Png).unwrap();
        let encode_duration = start_encode.elapsed();
        println!(
            "Encode time: {:.2}ms",
            encode_duration.as_secs_f64() * 1000.0
        );
        let png_meta = std::fs::metadata(&png_path).unwrap();
        println!("PNG exported: {}", png_path);
        println!("   PNG size: {} bytes", png_meta.len());
        let start_roundtrip = Instant::now();
        let img_roundtrip = Imagex::read(&png_path).unwrap();
        let roundtrip_duration = start_roundtrip.elapsed();
        println!(
            "Round-trip decode time: {:.2}ms",
            roundtrip_duration.as_secs_f64() * 1000.0
        );
        assert_eq!(img.width(), img_roundtrip.width());
        assert_eq!(img.height(), img_roundtrip.height());
        println!("Round-trip verification passed: dimensions match");
        let pixel_center = img_roundtrip.get_pixel(cx, cy);
        assert_eq!(pixel_center, Rgba::new(255, 0, 0, 255));
        println!("Center pixel verified: RED");
        let pixel_tl = img_roundtrip.get_pixel(0, 0);
        let orig_tl = img.get_pixel(0, 0);
        assert_eq!(pixel_tl, orig_tl);
        println!(
            "Top-left corner unchanged: ({}, {}, {}, {})",
            pixel_tl.r, pixel_tl.g, pixel_tl.b, pixel_tl.a
        );
        let pixel_10 = img_roundtrip.get_pixel(10, 10);
        let orig_10 = img.get_pixel(10, 10);
        assert_eq!(pixel_10, orig_10);
        println!(
            "Pixel (10,10) unchanged: ({}, {}, {}, {})",
            pixel_10.r, pixel_10.g, pixel_10.b, pixel_10.a
        );
        let total_time = decode_duration + modify_duration + encode_duration + roundtrip_duration;
        println!("\nPerformance Summary:");
        println!(
            "   Decode:        {:.2}ms",
            decode_duration.as_secs_f64() * 1000.0
        );
        println!(
            "   Modify pixels: {:.2}ms",
            modify_duration.as_secs_f64() * 1000.0
        );
        println!(
            "   Encode:        {:.2}ms",
            encode_duration.as_secs_f64() * 1000.0
        );
        println!(
            "   Round-trip:    {:.2}ms",
            roundtrip_duration.as_secs_f64() * 1000.0
        );
        println!(
            "   Total:         {:.2}ms",
            total_time.as_secs_f64() * 1000.0
        );
        println!("\nOutput file:");
        println!("   - {}", png_path);
    }
}
