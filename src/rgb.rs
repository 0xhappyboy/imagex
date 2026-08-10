/// RGB color channel structure with 8-bit precision
///
/// Represents a single pixel with Red, Green, Blue channels.
/// Each channel is stored as u8 (0-255).
///
/// # Memory Layout
/// ```
/// RGB { r, g, b }
/// ```
/// Total size: 3 bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl Rgb {
    /// Creates a new RGB pixel
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    /// Creates a grayscale RGB pixel (R=G=B)
    #[inline]
    pub const fn gray(value: u8) -> Self {
        Self {
            r: value,
            g: value,
            b: value,
        }
    }
    /// Creates a black pixel
    #[inline]
    pub const fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }
    /// Creates a white pixel
    #[inline]
    pub const fn white() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
        }
    }
    /// Converts to f32 channels (0.0 - 1.0)
    #[inline]
    pub fn to_f32(&self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }
    /// Converts from f32 channels (0.0 - 1.0)
    #[inline]
    pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }
    /// Gets the luminance (perceived brightness) of the pixel
    #[inline]
    pub fn luminance(&self) -> u8 {
        (self.r as f32 * 0.299 + self.g as f32 * 0.587 + self.b as f32 * 0.114) as u8
    }
    /// Linear interpolation between two RGB colors
    #[inline]
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: ((self.r as f64 + (other.r as f64 - self.r as f64) * t) as u8),
            g: ((self.g as f64 + (other.g as f64 - self.g as f64) * t) as u8),
            b: ((self.b as f64 + (other.b as f64 - self.b as f64) * t) as u8),
        }
    }
    /// Component-wise addition (saturating)
    #[inline]
    pub fn saturating_add(&self, other: &Self) -> Self {
        Self {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
        }
    }
    /// Component-wise subtraction (saturating)
    #[inline]
    pub fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            r: self.r.saturating_sub(other.r),
            g: self.g.saturating_sub(other.g),
            b: self.b.saturating_sub(other.b),
        }
    }
    /// Component-wise multiply
    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            r: ((self.r as f32 / 255.0) * (other.r as f32 / 255.0) * 255.0) as u8,
            g: ((self.g as f32 / 255.0) * (other.g as f32 / 255.0) * 255.0) as u8,
            b: ((self.b as f32 / 255.0) * (other.b as f32 / 255.0) * 255.0) as u8,
        }
    }
    /// Invert the colors
    #[inline]
    pub fn invert(&self) -> Self {
        Self {
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
        }
    }
    /// Gets the max channel value
    #[inline]
    pub fn max_channel(&self) -> u8 {
        self.r.max(self.g).max(self.b)
    }
    /// Gets the min channel value
    #[inline]
    pub fn min_channel(&self) -> u8 {
        self.r.min(self.g).min(self.b)
    }
    /// Apply a color tint
    #[inline]
    pub fn tint(&self, tint_color: &Self, amount: f64) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        self.lerp(tint_color, amount)
    }
}
impl From<[u8; 3]> for Rgb {
    #[inline]
    fn from(arr: [u8; 3]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
        }
    }
}
impl From<Rgb> for [u8; 3] {
    #[inline]
    fn from(pixel: Rgb) -> Self {
        [pixel.r, pixel.g, pixel.b]
    }
}
impl From<(u8, u8, u8)> for Rgb {
    #[inline]
    fn from(tuple: (u8, u8, u8)) -> Self {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
        }
    }
}
impl Default for Rgb {
    #[inline]
    fn default() -> Self {
        Self::black()
    }
}
