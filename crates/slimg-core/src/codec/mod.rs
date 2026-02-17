pub mod jpeg;
pub mod png;

use crate::error::Result;
use crate::format::Format;

/// Decoded image data in RGBA format (4 bytes per pixel).
#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl ImageData {
    /// Create a new `ImageData`.
    ///
    /// # Panics (debug only)
    /// Panics if `data.len()` does not equal `width * height * 4`.
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        debug_assert_eq!(
            data.len(),
            (width as usize) * (height as usize) * 4,
            "ImageData: expected {} bytes ({}x{}x4), got {}",
            (width as usize) * (height as usize) * 4,
            width,
            height,
            data.len(),
        );
        Self {
            width,
            height,
            data,
        }
    }

    /// Convert RGBA pixel data to RGB by dropping the alpha channel.
    pub fn to_rgb(&self) -> Vec<u8> {
        self.data
            .chunks_exact(4)
            .flat_map(|px| &px[..3])
            .copied()
            .collect()
    }
}

/// Options for encoding an image.
#[derive(Debug, Clone, Copy)]
pub struct EncodeOptions {
    /// Quality value in the range 0..=100.
    pub quality: u8,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self { quality: 80 }
    }
}

/// Trait implemented by each image codec (JPEG, PNG, WebP, etc.).
pub trait Codec {
    /// The image format handled by this codec.
    fn format(&self) -> Format;

    /// Decode raw file bytes into RGBA `ImageData`.
    fn decode(&self, data: &[u8]) -> Result<ImageData>;

    /// Encode `ImageData` into the codec's file format.
    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>>;
}

/// Return the appropriate codec for the given format.
pub fn get_codec(format: Format) -> Box<dyn Codec> {
    match format {
        Format::Jpeg => Box::new(jpeg::JpegCodec),
        Format::Png => Box::new(png::PngCodec),
        _ => unimplemented!("codec for {format:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_data_to_rgb() {
        // 2x1 image: red pixel (255,0,0,255) and green pixel (0,255,0,128)
        let data = vec![255, 0, 0, 255, 0, 255, 0, 128];
        let img = ImageData::new(2, 1, data);

        let rgb = img.to_rgb();
        assert_eq!(rgb, vec![255, 0, 0, 0, 255, 0]);
    }

    #[test]
    fn encode_options_default() {
        let opts = EncodeOptions::default();
        assert_eq!(opts.quality, 80);
    }

    #[test]
    fn image_data_dimensions() {
        let data = vec![0u8; 4 * 3 * 2]; // 3x2 image
        let img = ImageData::new(3, 2, data);
        assert_eq!(img.width, 3);
        assert_eq!(img.height, 2);
        assert_eq!(img.data.len(), 24);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ImageData: expected")]
    fn image_data_wrong_size_panics_in_debug() {
        let data = vec![0u8; 10]; // wrong size for any valid image
        let _ = ImageData::new(2, 2, data);
    }
}
