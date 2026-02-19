mod decoder;
mod encoder;
mod types;

use crate::error::Result;
use crate::format::Format;

use super::{Codec, EncodeOptions, ImageData};

/// JXL codec backed by libjxl (BSD-3-Clause) for both encoding and decoding.
pub struct JxlCodec;

impl Codec for JxlCodec {
    fn format(&self) -> Format {
        Format::Jxl
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let mut dec = decoder::Decoder::new()?;
        let (width, height, pixels) = dec.decode_to_rgba(data)?;
        Ok(ImageData::new(width, height, pixels))
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        let config = types::EncodeConfig::from_quality(options.quality);
        let mut enc = encoder::Encoder::new()?;
        enc.encode_rgba(&image.data, image.width, image.height, &config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let r = ((x * 255) / width.max(1)) as u8;
                let g = ((y * 255) / height.max(1)) as u8;
                let b = 128u8;
                let a = 255u8;
                pixels.extend_from_slice(&[r, g, b, a]);
            }
        }
        ImageData::new(width, height, pixels)
    }

    #[test]
    fn encode_lossy_produces_valid_jxl() {
        let codec = JxlCodec;
        let image = create_test_image(8, 8);
        let options = EncodeOptions { quality: 80 };

        let encoded = codec.encode(&image, &options).expect("encode should succeed");
        assert!(!encoded.is_empty(), "encoded data should not be empty");

        // Check JXL magic bytes (bare codestream: 0xFF 0x0A)
        assert!(
            (encoded.len() >= 2 && encoded[0] == 0xFF && encoded[1] == 0x0A)
                || (encoded.len() >= 8
                    && encoded[..4] == [0x00, 0x00, 0x00, 0x0C]
                    && &encoded[4..8] == b"JXL "),
            "output should have a valid JXL signature"
        );
    }

    #[test]
    fn encode_lossless_produces_valid_jxl() {
        let codec = JxlCodec;
        let image = create_test_image(8, 8);
        let options = EncodeOptions { quality: 100 };

        let encoded = codec.encode(&image, &options).expect("lossless encode should succeed");
        assert!(!encoded.is_empty());
    }

    #[test]
    fn roundtrip_lossy() {
        let codec = JxlCodec;
        let original = create_test_image(16, 16);
        let options = EncodeOptions { quality: 90 };

        let encoded = codec.encode(&original, &options).expect("encode failed");
        let decoded = codec.decode(&encoded).expect("decode failed");

        assert_eq!(decoded.width, original.width);
        assert_eq!(decoded.height, original.height);
        assert_eq!(decoded.data.len(), original.data.len());
    }

    #[test]
    fn roundtrip_lossless() {
        let codec = JxlCodec;
        let original = create_test_image(4, 4);
        let options = EncodeOptions { quality: 100 };

        let encoded = codec.encode(&original, &options).expect("encode failed");
        let decoded = codec.decode(&encoded).expect("decode failed");

        assert_eq!(decoded.width, original.width);
        assert_eq!(decoded.height, original.height);
        assert_eq!(
            decoded.data,
            original.data,
            "lossless roundtrip should produce identical pixels"
        );
    }
}
