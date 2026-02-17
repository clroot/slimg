use crate::error::{Error, Result};
use crate::format::Format;

use super::{Codec, EncodeOptions, ImageData};

/// JXL codec — decode-only (encoding is not supported due to license restrictions).
pub struct JxlCodec;

impl Codec for JxlCodec {
    fn format(&self) -> Format {
        Format::Jxl
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let img =
            image::load_from_memory(data).map_err(|e| Error::Decode(format!("jxl decode: {e}")))?;

        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();

        Ok(ImageData::new(width, height, rgba.into_raw()))
    }

    fn encode(&self, _image: &ImageData, _options: &EncodeOptions) -> Result<Vec<u8>> {
        Err(Error::EncodingNotSupported(Format::Jxl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_returns_not_supported() {
        let codec = JxlCodec;
        let data = vec![0u8; 4 * 2 * 2];
        let image = ImageData::new(2, 2, data);
        let options = EncodeOptions { quality: 80 };

        let result = codec.encode(&image, &options);
        assert!(result.is_err(), "JXL encoding should not be supported");
    }
}
