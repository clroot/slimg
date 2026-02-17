use crate::error::{Error, Result};
use crate::format::Format;

use super::{Codec, EncodeOptions, ImageData};

/// WebP codec backed by libwebp.
pub struct WebPCodec;

impl Codec for WebPCodec {
    fn format(&self) -> Format {
        Format::WebP
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let img = image::load_from_memory_with_format(data, image::ImageFormat::WebP)
            .map_err(|e| Error::Decode(format!("webp decode: {e}")))?;

        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();

        Ok(ImageData::new(width, height, rgba.into_raw()))
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        let encoder = webp::Encoder::from_rgba(&image.data, image.width, image.height);
        let encoded = encoder.encode(options.quality as f32);

        Ok(encoded.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let size = (width * height * 4) as usize;
        let mut data = vec![0u8; size];
        for y in 0..height {
            for x in 0..width {
                let i = ((y * width + x) * 4) as usize;
                data[i] = (x * 255 / width) as u8; // R
                data[i + 1] = (y * 255 / height) as u8; // G
                data[i + 2] = 128; // B
                data[i + 3] = 255; // A
            }
        }
        ImageData::new(width, height, data)
    }

    #[test]
    fn encode_and_decode_roundtrip() {
        let codec = WebPCodec;
        let original = create_test_image(64, 48);
        let options = EncodeOptions { quality: 90 };

        let encoded = codec.encode(&original, &options).expect("encode failed");

        // Decode back and verify dimensions
        let decoded = codec.decode(&encoded).expect("decode failed");
        assert_eq!(decoded.width, original.width);
        assert_eq!(decoded.height, original.height);
        assert_eq!(
            decoded.data.len(),
            (decoded.width * decoded.height * 4) as usize
        );
    }

    #[test]
    fn lower_quality_produces_smaller_file() {
        let codec = WebPCodec;
        let image = create_test_image(128, 96);

        let high = codec
            .encode(&image, &EncodeOptions { quality: 95 })
            .expect("encode q95 failed");
        let low = codec
            .encode(&image, &EncodeOptions { quality: 20 })
            .expect("encode q20 failed");

        assert!(
            low.len() < high.len(),
            "low quality ({} bytes) should be smaller than high quality ({} bytes)",
            low.len(),
            high.len(),
        );
    }
}
