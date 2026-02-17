use rapid_qoi::{Colors, Qoi};

use crate::error::{Error, Result};
use crate::format::Format;

use super::{Codec, EncodeOptions, ImageData};

/// QOI codec backed by rapid-qoi. Lossless format — quality is ignored.
pub struct QoiCodec;

impl Codec for QoiCodec {
    fn format(&self) -> Format {
        Format::Qoi
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let (header, pixels) =
            Qoi::decode_alloc(data).map_err(|e| Error::Decode(format!("qoi decode: {e}")))?;

        let width = header.width;
        let height = header.height;

        // Convert to RGBA if the decoded data is RGB (3 channels).
        let rgba = if header.colors.has_alpha() {
            pixels
        } else {
            pixels
                .chunks_exact(3)
                .flat_map(|px| [px[0], px[1], px[2], 255])
                .collect()
        };

        Ok(ImageData::new(width, height, rgba))
    }

    fn encode(&self, image: &ImageData, _options: &EncodeOptions) -> Result<Vec<u8>> {
        let qoi = Qoi {
            width: image.width,
            height: image.height,
            colors: Colors::SrgbLinA,
        };

        let encoded = qoi
            .encode_alloc(&image.data)
            .map_err(|e| Error::Encode(format!("qoi encode: {e}")))?;

        Ok(encoded)
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
        let codec = QoiCodec;
        let original = create_test_image(64, 48);
        let options = EncodeOptions { quality: 90 };

        let encoded = codec.encode(&original, &options).expect("encode failed");

        // Verify QOI magic bytes "qoif"
        assert!(
            encoded.len() >= 4,
            "encoded data too short: {} bytes",
            encoded.len()
        );
        assert_eq!(&encoded[..4], b"qoif", "missing QOI magic bytes");

        // Decode back and verify lossless roundtrip
        let decoded = codec.decode(&encoded).expect("decode failed");
        assert_eq!(decoded.width, original.width);
        assert_eq!(decoded.height, original.height);
        assert_eq!(decoded.data, original.data, "QOI should be lossless");
    }
}
