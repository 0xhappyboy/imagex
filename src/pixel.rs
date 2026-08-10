//! Pixel iterators for the image processing library.
//!
//! This module provides:
//! - `PixelIter`: Immutable pixel iterator for Imagex
//! - `PixelIterMut`: Mutable pixel iterator with proper lifetime handling
//! - `PixelRefMut`: Mutable reference to a single pixel's channels
use crate::{Rgb, Rgba};
use std::marker::PhantomData;
/// Immutable iterator over pixels in an Imagex
///
/// Iterates through all pixels in row-major order, returning each pixel
/// as an `Rgba` value. This is the safe, zero-copy way to read all pixels
/// from an image buffer.
///
/// # Example
/// ```
/// use cvx::Imagex;
///
/// let image = Imagex::new(1920, 1080);
/// for pixel in image.pixels() {
///     let r = pixel.r;
///     let g = pixel.g;
///     let b = pixel.b;
///     // Process pixel...
/// }
/// ```
pub struct PixelIter<'a> {
    /// Reference to the pixel data buffer
    pub data: &'a [u8],
    /// Number of bytes per row (including padding)
    pub stride: usize,
    /// Image width in pixels
    pub width: usize,
    /// Image height in pixels
    pub height: usize,
    /// Current x position in the iteration
    pub x: usize,
    /// Current y position in the iteration
    pub y: usize,
    /// Current index into the data buffer
    pub idx: usize,
}
impl<'a> Iterator for PixelIter<'a> {
    type Item = Rgba;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.height {
            return None;
        }
        // Read the pixel at the current position
        let pixel = Rgba {
            r: self.data[self.idx],
            g: self.data[self.idx + 1],
            b: self.data[self.idx + 2],
            a: self.data[self.idx + 3],
        };
        // Advance to the next pixel
        self.x += 1;
        self.idx += 4;
        // Move to the next row if we've reached the end of the current row
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
            self.idx = self.y * self.stride;
        }
        Some(pixel)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.height - self.y) * self.width - self.x;
        (remaining, Some(remaining))
    }
}
impl<'a> ExactSizeIterator for PixelIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        (self.height - self.y) * self.width - self.x
    }
}
/// Mutable iterator over pixels in an Imagex
///
/// Iterates through all pixels in row-major order, returning a `PixelRefMut`
/// for each pixel. This allows in-place modification of pixel data without
/// copying.
///
/// # Safety
/// This iterator uses raw pointers internally for performance. The `PhantomData`
/// ensures proper lifetime tracking so the borrow checker can validate safety.
///
/// # Example
/// ```
/// use cvx::Imagex;
///
/// let mut image = Imagex::new(1920, 1080);
/// for mut pixel in image.pixels_mut() {
///     // Double the red channel
///     *pixel.r = (*pixel.r as f32 * 2.0).min(255.0) as u8;
/// }
/// ```
pub struct PixelIterMut<'a> {
    /// Raw pointer to the pixel data buffer
    pub ptr: *mut u8,
    /// Number of bytes per row (including padding)
    pub stride: usize,
    /// Image width in pixels
    pub width: usize,
    /// Image height in pixels
    pub height: usize,
    /// Current x position in the iteration
    pub x: usize,
    /// Current y position in the iteration
    pub y: usize,
    /// Current index into the data buffer
    pub idx: usize,
    /// Marker to track the lifetime of the borrowed data
    pub _marker: PhantomData<&'a mut u8>,
}
impl<'a> Iterator for PixelIterMut<'a> {
    type Item = PixelRefMut<'a>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.height {
            return None;
        }
        // SAFETY: We're iterating within bounds.
        // The iterator ensures idx never exceeds the buffer size.
        // The lifetime is tracked via PhantomData.
        let pixel = unsafe {
            PixelRefMut {
                r: &mut *self.ptr.add(self.idx),
                g: &mut *self.ptr.add(self.idx + 1),
                b: &mut *self.ptr.add(self.idx + 2),
                a: &mut *self.ptr.add(self.idx + 3),
            }
        };
        // Advance to the next pixel
        self.x += 1;
        self.idx += 4;
        // Move to the next row if we've reached the end of the current row
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
            self.idx = self.y * self.stride;
        }
        Some(pixel)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.height - self.y) * self.width - self.x;
        (remaining, Some(remaining))
    }
}
impl<'a> ExactSizeIterator for PixelIterMut<'a> {
    #[inline]
    fn len(&self) -> usize {
        (self.height - self.y) * self.width - self.x
    }
}
/// Mutable reference to a single pixel's color channels
///
/// Provides direct mutable access to the R, G, B, A channels of a pixel.
/// This is returned by `PixelIterMut` to allow in-place modification.
///
/// # Example
/// ```
/// use imagex::{Imagex, Rgba};
///
/// let mut image = Imagex::new(100, 100);
/// for mut pixel in image.pixels_mut() {
///     // Set all pixels to white
///     pixel.set(Rgba::white_opaque());
/// }
/// ```
pub struct PixelRefMut<'a> {
    /// Mutable reference to the red channel (0-255)
    pub r: &'a mut u8,
    /// Mutable reference to the green channel (0-255)
    pub g: &'a mut u8,
    /// Mutable reference to the blue channel (0-255)
    pub b: &'a mut u8,
    /// Mutable reference to the alpha channel (0-255)
    pub a: &'a mut u8,
}
impl<'a> PixelRefMut<'a> {
    /// Sets the pixel to a new value
    ///
    /// # Example
    /// ```
    /// use imagex::{Imagex, Rgba};
    ///
    /// let mut image = Imagex::new(100, 100);
    /// for mut pixel in image.pixels_mut() {
    ///     pixel.set(Rgba::white_opaque());
    /// }
    /// ```
    #[inline]
    pub fn set(&mut self, pixel: Rgba) {
        *self.r = pixel.r;
        *self.g = pixel.g;
        *self.b = pixel.b;
        *self.a = pixel.a;
    }
    /// Converts the mutable reference to an RGB pixel (ignores alpha)
    #[inline]
    pub fn to_rgb(&self) -> Rgb {
        Rgb {
            r: *self.r,
            g: *self.g,
            b: *self.b,
        }
    }
    /// Converts the mutable reference to an RGBA pixel
    #[inline]
    pub fn to_rgba(&self) -> Rgba {
        Rgba {
            r: *self.r,
            g: *self.g,
            b: *self.b,
            a: *self.a,
        }
    }
}
