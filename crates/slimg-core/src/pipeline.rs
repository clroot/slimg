use std::fs;
use std::path::{Path, PathBuf};

use crate::codec::{EncodeOptions, ImageData, get_codec};
use crate::error::{Error, Result};
use crate::format::Format;
use crate::crop::{self, CropMode};
use crate::resize::{self, ResizeMode};

/// Options for a conversion pipeline.
#[derive(Debug, Clone)]
pub struct PipelineOptions {
    /// Target output format.
    pub format: Format,
    /// Encoding quality (0..=100).
    pub quality: u8,
    /// Optional resize to apply before encoding.
    pub resize: Option<ResizeMode>,
    /// Optional crop to apply before encoding.
    pub crop: Option<CropMode>,
}

/// Result of a pipeline conversion.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Encoded image bytes.
    pub data: Vec<u8>,
    /// Format of the encoded data.
    pub format: Format,
}

impl PipelineResult {
    /// Write the encoded data to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        fs::write(path, &self.data)?;
        Ok(())
    }
}

/// Detect the format from magic bytes and decode the raw image data.
pub fn decode(data: &[u8]) -> Result<(ImageData, Format)> {
    let format = Format::from_magic_bytes(data)
        .ok_or_else(|| Error::UnknownFormat("unrecognised magic bytes".to_string()))?;

    let codec = get_codec(format);
    let image = codec.decode(data)?;
    Ok((image, format))
}

/// Read a file from disk, detect its format, and decode it.
pub fn decode_file(path: &Path) -> Result<(ImageData, Format)> {
    let data = fs::read(path)?;
    decode(&data)
}

/// Convert an image to the specified format, optionally resizing first.
pub fn convert(image: &ImageData, options: &PipelineOptions) -> Result<PipelineResult> {
    if !options.format.can_encode() {
        return Err(Error::EncodingNotSupported(options.format));
    }

    let image = match &options.crop {
        Some(mode) => crop::crop(image, mode)?,
        None => image.clone(),
    };

    let image = match &options.resize {
        Some(mode) => resize::resize(&image, mode)?,
        None => image,
    };

    let codec = get_codec(options.format);
    let encode_opts = EncodeOptions {
        quality: options.quality,
    };
    let data = codec.encode(&image, &encode_opts)?;

    Ok(PipelineResult {
        data,
        format: options.format,
    })
}

/// Decode the data and re-encode in the same format at the given quality.
pub fn optimize(data: &[u8], quality: u8) -> Result<PipelineResult> {
    let (image, format) = decode(data)?;

    if !format.can_encode() {
        return Err(Error::EncodingNotSupported(format));
    }

    let codec = get_codec(format);
    let encode_opts = EncodeOptions { quality };
    let encoded = codec.encode(&image, &encode_opts)?;

    Ok(PipelineResult {
        data: encoded,
        format,
    })
}

/// Derive an output path for the converted image.
///
/// - If `output` is `None`, uses the input directory with the new extension.
/// - If `output` is a directory, places the file there with the new extension.
/// - Otherwise, uses `output` as-is.
pub fn output_path(input: &Path, format: Format, output: Option<&Path>) -> PathBuf {
    let new_ext = format.extension();

    match output {
        None => input.with_extension(new_ext),
        Some(out) if out.is_dir() => {
            let stem = input.file_stem().unwrap_or_default();
            out.join(stem).with_extension(new_ext)
        }
        Some(out) => out.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn output_path_changes_extension() {
        let result = output_path(Path::new("/tmp/photo.jpg"), Format::WebP, None);
        assert_eq!(result, PathBuf::from("/tmp/photo.webp"));
    }

    #[test]
    fn output_path_with_explicit_output() {
        let result = output_path(
            Path::new("/tmp/photo.jpg"),
            Format::Png,
            Some(Path::new("/out/result.png")),
        );
        assert_eq!(result, PathBuf::from("/out/result.png"));
    }

    #[test]
    fn jxl_encode_returns_error() {
        let image = ImageData::new(2, 2, vec![128u8; 16]);
        let options = PipelineOptions {
            format: Format::Jxl,
            quality: 80,
            resize: None,
            crop: None,
        };
        let result = convert(&image, &options);
        assert!(result.is_err(), "converting to JXL should fail");
    }
}
