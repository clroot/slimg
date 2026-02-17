use imgref::Img;
use rgb::RGBA8;

use crate::error::{Error, Result};
use crate::format::Format;

use super::{Codec, EncodeOptions, ImageData};

/// AVIF codec backed by ravif for encoding and the `image` crate for decoding.
pub struct AvifCodec;

impl Codec for AvifCodec {
    fn format(&self) -> Format {
        Format::Avif
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let img = image::load_from_memory_with_format(data, image::ImageFormat::Avif)
            .map_err(|e| Error::Decode(format!("avif decode: {e}")))?;

        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();

        Ok(ImageData::new(width, height, rgba.into_raw()))
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        let width = image.width as usize;
        let height = image.height as usize;

        // Convert raw RGBA bytes to Vec<RGBA8>
        let pixels: Vec<RGBA8> = image
            .data
            .chunks_exact(4)
            .map(|px| RGBA8::new(px[0], px[1], px[2], px[3]))
            .collect();

        let buffer = Img::new(pixels.as_slice(), width, height);

        let encoded = ravif::Encoder::new()
            .with_quality(options.quality as f32)
            .with_speed(6)
            .encode_rgba(buffer)
            .map_err(|e| Error::Encode(format!("ravif encode: {e}")))?;

        Ok(encoded.avif_file)
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
    fn encode_produces_valid_avif() {
        let codec = AvifCodec;
        let image = create_test_image(64, 48);
        let options = EncodeOptions { quality: 80 };

        let encoded = codec.encode(&image, &options).expect("encode failed");

        // Verify AVIF container: bytes 4-7 should be "ftyp"
        assert!(
            encoded.len() >= 8,
            "encoded data too short: {} bytes",
            encoded.len()
        );
        assert_eq!(&encoded[4..8], b"ftyp", "missing AVIF ftyp box");
    }

    #[test]
    fn encode_and_decode_roundtrip() {
        let codec = AvifCodec;
        let original = create_test_image(64, 48);
        let options = EncodeOptions { quality: 80 };

        let encoded = codec.encode(&original, &options).expect("encode failed");
        let decoded = codec.decode(&encoded).expect("decode failed");

        assert_eq!(decoded.width, original.width);
        assert_eq!(decoded.height, original.height);
        assert_eq!(
            decoded.data.len(),
            (decoded.width * decoded.height * 4) as usize
        );
    }
}
