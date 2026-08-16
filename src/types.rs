#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Unknown,
    Png,
    Jpeg,
    WebP,
    Bmp,
    Gif,
    Tiff,
    Ico,
    Avif,
    Ppm,
}
impl ImageFormat {
    pub fn from_path(path: &str) -> Self {
        let path = std::path::Path::new(path);
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
        {
            Some(ext) if ext == "png" => Self::Png,
            Some(ext) if ext == "jpg" || ext == "jpeg" => Self::Jpeg,
            Some(ext) if ext == "webp" => Self::WebP,
            Some(ext) if ext == "bmp" => Self::Bmp,
            Some(ext) if ext == "gif" => Self::Gif,
            Some(ext) if ext == "tiff" || ext == "tif" => Self::Tiff,
            Some(ext) if ext == "ico" => Self::Ico,
            Some(ext) if ext == "avif" => Self::Avif,
            Some(ext) if ext == "ppm" => Self::Ppm,
            _ => Self::Unknown,
        }
    }
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Bmp => "bmp",
            Self::Gif => "gif",
            Self::Tiff => "tiff",
            Self::Ico => "ico",
            Self::Avif => "avif",
            Self::Ppm => "ppm",
            Self::Unknown => "",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Rgb,
    Rgba,
    Grayscale,
    GrayscaleAlpha,
    Cmyk,
    Unknown,
}
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub color_space: ColorSpace,
    pub bit_depth: u8,
    pub has_alpha: bool,
    pub compression: Option<String>,
}
impl ImageInfo {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            format: ImageFormat::Unknown,
            color_space: ColorSpace::Rgba,
            bit_depth: 8,
            has_alpha: true,
            compression: None,
        }
    }
}
