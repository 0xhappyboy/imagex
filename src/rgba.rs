use crate::Rgb;
/// RGBA color channel structure with 8-bit precision
///
/// Represents a single pixel with Red, Green, Blue, and Alpha channels.
/// Alpha channel controls transparency (255 = fully opaque, 0 = transparent).
///
/// # Memory Layout
/// ```
/// Rgba { r, g, b, a }
/// ```
/// Total size: 4 bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Rgba {
    /// Creates a new RGBA pixel
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    /// Creates an RGB pixel with full opacity
    #[inline]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    /// Creates a grayscale RGBA pixel
    #[inline]
    pub const fn gray(value: u8) -> Self {
        Self {
            r: value,
            g: value,
            b: value,
            a: 255,
        }
    }
    /// Creates a black pixel with full opacity
    #[inline]
    pub const fn black_opaque() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
    /// Creates a white pixel with full opacity
    #[inline]
    pub const fn white_opaque() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
    /// Creates a fully transparent pixel
    #[inline]
    pub const fn transparent() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
    /// Converts to f32 channels (0.0 - 1.0)
    #[inline]
    pub fn to_f32(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
    /// Converts from f32 channels (0.0 - 1.0)
    #[inline]
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
            a: (a.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }
    /// Drops the alpha channel, returns RGB
    #[inline]
    pub fn to_rgb(&self) -> Rgb {
        Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
    /// Linear interpolation between two RGBA colors
    #[inline]
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: ((self.r as f64 + (other.r as f64 - self.r as f64) * t) as u8),
            g: ((self.g as f64 + (other.g as f64 - self.g as f64) * t) as u8),
            b: ((self.b as f64 + (other.b as f64 - self.b as f64) * t) as u8),
            a: ((self.a as f64 + (other.a as f64 - self.a as f64) * t) as u8),
        }
    }
    /// Alpha compositing: blend with another color
    #[inline]
    pub fn composite(&self, background: &Self) -> Self {
        let a = self.a as f32 / 255.0;
        let bg_a = background.a as f32 / 255.0;
        let out_a = a + bg_a * (1.0 - a);
        if out_a <= 0.0 {
            return Self::transparent();
        }
        let r = (self.r as f32 * a + background.r as f32 * bg_a * (1.0 - a)) / out_a;
        let g = (self.g as f32 * a + background.g as f32 * bg_a * (1.0 - a)) / out_a;
        let b = (self.b as f32 * a + background.b as f32 * bg_a * (1.0 - a)) / out_a;
        Self {
            r: r.clamp(0.0, 255.0) as u8,
            g: g.clamp(0.0, 255.0) as u8,
            b: b.clamp(0.0, 255.0) as u8,
            a: (out_a * 255.0) as u8,
        }
    }
}
impl From<Rgb> for Rgba {
    #[inline]
    fn from(rgb: Rgb) -> Self {
        Self {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
            a: 255,
        }
    }
}
impl From<Rgba> for [u8; 4] {
    #[inline]
    fn from(pixel: Rgba) -> Self {
        [pixel.r, pixel.g, pixel.b, pixel.a]
    }
}
impl From<(u8, u8, u8, u8)> for Rgba {
    #[inline]
    fn from(tuple: (u8, u8, u8, u8)) -> Self {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
            a: tuple.3,
        }
    }
}
impl Default for Rgba {
    #[inline]
    fn default() -> Self {
        Self::black_opaque()
    }
}
