use crate::error::{Error, Result};
use crate::format::Format;

use super::{Codec, EncodeOptions, ImageData};

/// JPEG codec backed by MozJPEG.
pub struct JpegCodec;

impl Codec for JpegCodec {
    fn format(&self) -> Format {
        Format::Jpeg
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        // mozjpeg uses setjmp/longjmp internally, which translates to panics
        // in Rust. We must catch those to turn them into proper errors.
        let data = data.to_vec();
        let result = std::panic::catch_unwind(move || -> Result<ImageData> {
            let decompress = mozjpeg::Decompress::new_mem(&data)
                .map_err(|e| Error::Decode(format!("mozjpeg decompress init: {e}")))?;

            let width = decompress.width() as u32;
            let height = decompress.height() as u32;

            let mut decompressor = decompress
                .rgba()
                .map_err(|e| Error::Decode(format!("mozjpeg rgba conversion: {e}")))?;

            let pixels: Vec<[u8; 4]> = decompressor
                .read_scanlines()
                .map_err(|e| Error::Decode(format!("mozjpeg read scanlines: {e}")))?;

            decompressor
                .finish()
                .map_err(|e| Error::Decode(format!("mozjpeg finish: {e}")))?;

            let rgba_data: Vec<u8> = pixels.into_iter().flatten().collect();

            Ok(ImageData::new(width, height, rgba_data))
        });

        match result {
            Ok(inner) => inner,
            Err(panic) => {
                let msg = panic_message(&panic);
                Err(Error::Decode(format!("mozjpeg panicked: {msg}")))
            }
        }
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        let width = image.width;
        let height = image.height;
        let rgb_data = image.to_rgb();
        let quality = options.quality as f32;

        let result = std::panic::catch_unwind(move || -> Result<Vec<u8>> {
            let mut compress = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);

            compress.set_size(width as usize, height as usize);
            compress.set_quality(quality);
            compress.set_progressive_mode();
            compress.set_optimize_scans(true);
            compress.set_optimize_coding(true);

            let mut compressor = compress
                .start_compress(Vec::new())
                .map_err(|e| Error::Encode(format!("mozjpeg compress start: {e}")))?;

            compressor
                .write_scanlines(&rgb_data)
                .map_err(|e| Error::Encode(format!("mozjpeg write scanlines: {e}")))?;

            let output = compressor
                .finish()
                .map_err(|e| Error::Encode(format!("mozjpeg finish: {e}")))?;

            Ok(output)
        });

        match result {
            Ok(inner) => inner,
            Err(panic) => {
                let msg = panic_message(&panic);
                Err(Error::Encode(format!("mozjpeg panicked: {msg}")))
            }
        }
    }
}

/// Extract a human-readable message from a `catch_unwind` panic payload.
fn panic_message(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
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
        let codec = JpegCodec;
        let original = create_test_image(64, 48);
        let options = EncodeOptions { quality: 90 };

        let encoded = codec.encode(&original, &options).expect("encode failed");

        // Verify JPEG magic bytes
        assert!(
            encoded.len() >= 3,
            "encoded data too short: {} bytes",
            encoded.len()
        );
        assert_eq!(
            &encoded[..3],
            &[0xFF, 0xD8, 0xFF],
            "missing JPEG magic bytes"
        );

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
    fn encode_produces_smaller_at_lower_quality() {
        let codec = JpegCodec;
        let image = create_test_image(128, 96);

        let high = codec
            .encode(&image, &EncodeOptions { quality: 95 })
            .expect("encode q95 failed");
        let low = codec
            .encode(&image, &EncodeOptions { quality: 30 })
            .expect("encode q30 failed");

        assert!(
            low.len() < high.len(),
            "low quality ({} bytes) should be smaller than high quality ({} bytes)",
            low.len(),
            high.len(),
        );
    }

    #[test]
    fn decode_invalid_data_returns_error() {
        let codec = JpegCodec;
        let result = codec.decode(b"not a jpeg");
        assert!(result.is_err(), "decoding invalid data should fail");
    }
}
