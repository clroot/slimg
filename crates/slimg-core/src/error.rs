use crate::format::Format;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported format: {0:?}")]
    UnsupportedFormat(Format),

    #[error("unknown format for file: {0}")]
    UnknownFormat(String),

    #[error("encoding not supported for {0:?} (license restriction)")]
    EncodingNotSupported(Format),

    #[error("decode error: {0}")]
    Decode(String),

    #[error("encode error: {0}")]
    Encode(String),

    #[error("resize error: {0}")]
    Resize(String),

    #[error("crop error: {0}")]
    Crop(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),
}

pub type Result<T> = std::result::Result<T, Error>;
