uniffi::setup_scaffolding!();

use std::path::Path;

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum Format {
    Jpeg,
    Png,
    WebP,
    Avif,
    Jxl,
    Qoi,
}

impl Format {
    fn to_core(self) -> slimg_core::Format {
        match self {
            Format::Jpeg => slimg_core::Format::Jpeg,
            Format::Png => slimg_core::Format::Png,
            Format::WebP => slimg_core::Format::WebP,
            Format::Avif => slimg_core::Format::Avif,
            Format::Jxl => slimg_core::Format::Jxl,
            Format::Qoi => slimg_core::Format::Qoi,
        }
    }

    fn from_core(format: slimg_core::Format) -> Self {
        match format {
            slimg_core::Format::Jpeg => Format::Jpeg,
            slimg_core::Format::Png => Format::Png,
            slimg_core::Format::WebP => Format::WebP,
            slimg_core::Format::Avif => Format::Avif,
            slimg_core::Format::Jxl => Format::Jxl,
            slimg_core::Format::Qoi => Format::Qoi,
        }
    }
}

/// How to resize an image.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum ResizeMode {
    /// Set width, calculate height preserving aspect ratio.
    Width { value: u32 },
    /// Set height, calculate width preserving aspect ratio.
    Height { value: u32 },
    /// Exact dimensions (may distort the image).
    Exact { width: u32, height: u32 },
    /// Fit within bounds, preserving aspect ratio.
    Fit { max_width: u32, max_height: u32 },
    /// Scale factor (e.g. 0.5 = half size).
    Scale { factor: f64 },
}

impl ResizeMode {
    fn to_core(&self) -> slimg_core::ResizeMode {
        match self {
            ResizeMode::Width { value } => slimg_core::ResizeMode::Width(*value),
            ResizeMode::Height { value } => slimg_core::ResizeMode::Height(*value),
            ResizeMode::Exact { width, height } => slimg_core::ResizeMode::Exact(*width, *height),
            ResizeMode::Fit {
                max_width,
                max_height,
            } => slimg_core::ResizeMode::Fit(*max_width, *max_height),
            ResizeMode::Scale { factor } => slimg_core::ResizeMode::Scale(*factor),
        }
    }
}

/// How to crop an image.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum CropMode {
    /// Extract a specific region.
    Region { x: u32, y: u32, width: u32, height: u32 },
    /// Crop to an aspect ratio (centered).
    AspectRatio { width: u32, height: u32 },
}

impl CropMode {
    fn to_core(&self) -> slimg_core::CropMode {
        match self {
            CropMode::Region { x, y, width, height } => slimg_core::CropMode::Region {
                x: *x, y: *y, width: *width, height: *height,
            },
            CropMode::AspectRatio { width, height } => slimg_core::CropMode::AspectRatio {
                width: *width, height: *height,
            },
        }
    }
}

/// How to extend (add padding to) an image.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum ExtendMode {
    /// Extend the canvas so the image fits the given aspect ratio (centered).
    AspectRatio { width: u32, height: u32 },
    /// Extend the canvas to an exact pixel size (centered).
    Size { width: u32, height: u32 },
}

impl ExtendMode {
    fn to_core(&self) -> slimg_core::ExtendMode {
        match self {
            ExtendMode::AspectRatio { width, height } => slimg_core::ExtendMode::AspectRatio {
                width: *width,
                height: *height,
            },
            ExtendMode::Size { width, height } => slimg_core::ExtendMode::Size {
                width: *width,
                height: *height,
            },
        }
    }
}

/// Fill color for the extended canvas region.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum FillColor {
    /// A solid RGBA color.
    Solid { r: u8, g: u8, b: u8, a: u8 },
    /// Fully transparent (RGBA 0,0,0,0).
    Transparent,
}

impl FillColor {
    fn to_core(&self) -> slimg_core::FillColor {
        match self {
            FillColor::Solid { r, g, b, a } => slimg_core::FillColor::Solid([*r, *g, *b, *a]),
            FillColor::Transparent => slimg_core::FillColor::Transparent,
        }
    }
}

/// Decoded image data in RGBA format (4 bytes per pixel).
#[derive(Debug, Clone, uniffi::Record)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl ImageData {
    fn to_core(&self) -> slimg_core::ImageData {
        slimg_core::ImageData::new(self.width, self.height, self.data.clone())
    }

    fn from_core(img: slimg_core::ImageData) -> Self {
        Self {
            width: img.width,
            height: img.height,
            data: img.data,
        }
    }
}

/// Options for a conversion pipeline.
#[derive(Debug, Clone, uniffi::Record)]
pub struct PipelineOptions {
    /// Target output format.
    pub format: Format,
    /// Encoding quality (0-100).
    pub quality: u8,
    /// Optional resize to apply before encoding.
    pub resize: Option<ResizeMode>,
    /// Optional crop to apply before encoding.
    pub crop: Option<CropMode>,
    /// Optional extend (padding) to apply after crop and before resize.
    pub extend: Option<ExtendMode>,
    /// Fill color for the extended region (defaults to opaque white).
    pub fill_color: Option<FillColor>,
}

/// Result of a pipeline conversion.
#[derive(Debug, Clone, uniffi::Record)]
pub struct PipelineResult {
    /// Encoded image bytes.
    pub data: Vec<u8>,
    /// Format of the encoded data.
    pub format: Format,
}

/// Result of a decode operation.
#[derive(Debug, Clone, uniffi::Record)]
pub struct DecodeResult {
    /// Decoded RGBA image data.
    pub image: ImageData,
    /// Detected format of the input.
    pub format: Format,
}

/// Errors from slimg operations.
#[derive(Debug, uniffi::Error, thiserror::Error)]
pub enum SlimgError {
    #[error("unsupported format: {format}")]
    UnsupportedFormat { format: String },

    #[error("unknown format: {detail}")]
    UnknownFormat { detail: String },

    #[error("encoding not supported: {format}")]
    EncodingNotSupported { format: String },

    #[error("decode error: {message}")]
    Decode { message: String },

    #[error("encode error: {message}")]
    Encode { message: String },

    #[error("resize error: {message}")]
    Resize { message: String },

    #[error("crop error: {message}")]
    Crop { message: String },

    #[error("extend error: {message}")]
    Extend { message: String },

    #[error("I/O error: {message}")]
    Io { message: String },

    #[error("image error: {message}")]
    Image { message: String },
}

impl From<slimg_core::Error> for SlimgError {
    fn from(e: slimg_core::Error) -> Self {
        match e {
            slimg_core::Error::UnsupportedFormat(f) => SlimgError::UnsupportedFormat {
                format: format!("{f:?}"),
            },
            slimg_core::Error::UnknownFormat(s) => SlimgError::UnknownFormat { detail: s },
            slimg_core::Error::EncodingNotSupported(f) => SlimgError::EncodingNotSupported {
                format: format!("{f:?}"),
            },
            slimg_core::Error::Decode(s) => SlimgError::Decode { message: s },
            slimg_core::Error::Encode(s) => SlimgError::Encode { message: s },
            slimg_core::Error::Resize(s) => SlimgError::Resize { message: s },
            slimg_core::Error::Crop(s) => SlimgError::Crop { message: s },
            slimg_core::Error::Extend(s) => SlimgError::Extend { message: s },
            slimg_core::Error::Io(e) => SlimgError::Io {
                message: e.to_string(),
            },
            slimg_core::Error::Image(e) => SlimgError::Image {
                message: e.to_string(),
            },
        }
    }
}

// ── Exported functions ───────────────────────────────────

/// Returns the canonical file extension for the given format.
#[uniffi::export]
fn format_extension(format: Format) -> String {
    format.to_core().extension().to_string()
}

/// Whether encoding is supported for the given format.
#[uniffi::export]
fn format_can_encode(format: Format) -> bool {
    format.to_core().can_encode()
}

/// Detect format from file extension (case-insensitive).
#[uniffi::export]
fn format_from_extension(path: String) -> Option<Format> {
    slimg_core::Format::from_extension(Path::new(&path)).map(Format::from_core)
}

/// Detect format from magic bytes at the start of file data.
#[uniffi::export]
fn format_from_magic_bytes(data: Vec<u8>) -> Option<Format> {
    slimg_core::Format::from_magic_bytes(&data).map(Format::from_core)
}

/// Detect the format from magic bytes and decode raw image data.
#[uniffi::export]
fn decode(data: Vec<u8>) -> Result<DecodeResult, SlimgError> {
    let (image, format) = slimg_core::decode(&data)?;
    Ok(DecodeResult {
        image: ImageData::from_core(image),
        format: Format::from_core(format),
    })
}

/// Read a file from disk, detect its format, and decode it.
#[uniffi::export]
fn decode_file(path: String) -> Result<DecodeResult, SlimgError> {
    let (image, format) = slimg_core::decode_file(Path::new(&path))?;
    Ok(DecodeResult {
        image: ImageData::from_core(image),
        format: Format::from_core(format),
    })
}

/// Convert an image to the specified format, optionally resizing first.
#[uniffi::export]
fn convert(image: &ImageData, options: &PipelineOptions) -> Result<PipelineResult, SlimgError> {
    let core_options = slimg_core::PipelineOptions {
        format: options.format.to_core(),
        quality: options.quality,
        resize: options.resize.as_ref().map(|r| r.to_core()),
        crop: options.crop.as_ref().map(|c| c.to_core()),
        extend: options.extend.as_ref().map(|e| e.to_core()),
        fill_color: options.fill_color.as_ref().map(|f| f.to_core()),
    };
    let result = slimg_core::convert(&image.to_core(), &core_options)?;
    Ok(PipelineResult {
        data: result.data,
        format: Format::from_core(result.format),
    })
}

/// Crop an image according to the given mode.
#[uniffi::export]
fn crop(image: &ImageData, mode: &CropMode) -> Result<ImageData, SlimgError> {
    let result = slimg_core::crop::crop(&image.to_core(), &mode.to_core())?;
    Ok(ImageData::from_core(result))
}

/// Extend an image by adding padding around it.
#[uniffi::export]
fn extend(image: &ImageData, mode: &ExtendMode, fill: &FillColor) -> Result<ImageData, SlimgError> {
    let result = slimg_core::extend::extend(&image.to_core(), &mode.to_core(), &fill.to_core())?;
    Ok(ImageData::from_core(result))
}

/// Resize an image according to the given mode.
#[uniffi::export]
fn resize(image: &ImageData, mode: &ResizeMode) -> Result<ImageData, SlimgError> {
    let result = slimg_core::resize::resize(&image.to_core(), &mode.to_core())?;
    Ok(ImageData::from_core(result))
}

/// Decode the data and re-encode in the same format at the given quality.
#[uniffi::export]
fn optimize(data: Vec<u8>, quality: u8) -> Result<PipelineResult, SlimgError> {
    let result = slimg_core::optimize(&data, quality)?;
    Ok(PipelineResult {
        data: result.data,
        format: Format::from_core(result.format),
    })
}

/// Derive an output path for the converted image.
#[uniffi::export]
fn output_path(input: String, format: Format, output: Option<String>) -> String {
    let result = slimg_core::output_path(
        Path::new(&input),
        format.to_core(),
        output.as_ref().map(|s| Path::new(s.as_str())),
    );
    result.to_string_lossy().to_string()
}
