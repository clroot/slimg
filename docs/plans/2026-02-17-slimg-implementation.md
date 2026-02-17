# slimg Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Squoosh급 이미지 압축/최적화/포맷 변환을 제공하는 Rust CLI + 라이브러리를 구현한다.

**Architecture:** 멀티 크레이트 워크스페이스. `slimg-core`가 코덱/파이프라인/리사이즈 로직을 제공하고, `cli`가 clap 기반 서브커맨드 인터페이스를 제공한다. 각 코덱은 `Codec` trait을 구현하여 일관된 encode/decode 인터페이스를 갖는다.

**Tech Stack:** Rust 2024 edition, clap 4.5 (derive), mozjpeg 0.10, webp 0.3, ravif 0.13, oxipng 10.1, jxl-oxide 0.12, rapid-qoi 0.6, image 0.25

---

## Task 1: Workspace Scaffolding

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/slimg-core/Cargo.toml`
- Create: `crates/slimg-core/src/lib.rs`
- Create: `cli/Cargo.toml`
- Create: `cli/src/main.rs`
- Create: `.gitignore`

**Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/slimg-core", "cli"]
```

**Step 2: Create slimg-core crate**

`crates/slimg-core/Cargo.toml`:
```toml
[package]
name = "slimg-core"
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"
description = "Image optimization library — encode, decode, convert, resize"

[dependencies]
image = "0.25"
thiserror = "2"
```

`crates/slimg-core/src/lib.rs`:
```rust
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

**Step 3: Create CLI crate**

`cli/Cargo.toml`:
```toml
[package]
name = "slimg"
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"
description = "Image optimization CLI — convert, optimize, resize"

[dependencies]
slimg-core = { path = "../crates/slimg-core" }
clap = { version = "4.5", features = ["derive"] }
anyhow = "1"
```

`cli/src/main.rs`:
```rust
fn main() {
    println!("slimg v{}", slimg_core::version());
}
```

**Step 4: Create .gitignore**

```
/target
```

**Step 5: Verify build**

Run: `cargo build`
Expected: Compiles successfully

Run: `cargo run -p slimg`
Expected: `slimg v0.1.0`

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: scaffold workspace with slimg-core and cli crates"
```

---

## Task 2: Core Types & Codec Trait

**Files:**
- Create: `crates/slimg-core/src/format.rs`
- Create: `crates/slimg-core/src/codec.rs`
- Create: `crates/slimg-core/src/error.rs`
- Modify: `crates/slimg-core/src/lib.rs`

**Step 1: Define Format enum**

`crates/slimg-core/src/format.rs`:
```rust
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Jpeg,
    Png,
    WebP,
    Avif,
    Jxl,
    Qoi,
}

impl Format {
    /// Detect format from file extension.
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::WebP),
            "avif" => Some(Self::Avif),
            "jxl" => Some(Self::Jxl),
            "qoi" => Some(Self::Qoi),
            _ => None,
        }
    }

    /// Detect format from magic bytes.
    pub fn from_magic_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 12 {
            return None;
        }
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(Self::Jpeg);
        }
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Some(Self::Png);
        }
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return Some(Self::WebP);
        }
        if data.len() >= 12
            && &data[4..8] == b"ftyp"
            && (data[8..12].starts_with(b"avif") || data[8..12].starts_with(b"avis"))
        {
            return Some(Self::Avif);
        }
        if data.starts_with(&[0xFF, 0x0A])
            || (data.len() >= 12
                && &data[0..4] == [0x00, 0x00, 0x00, 0x0C]
                && &data[4..8] == b"JXL ")
        {
            return Some(Self::Jxl);
        }
        if data.starts_with(b"qoif") {
            return Some(Self::Qoi);
        }
        None
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Avif => "avif",
            Self::Jxl => "jxl",
            Self::Qoi => "qoi",
        }
    }

    pub fn can_encode(&self) -> bool {
        // JXL encoding requires GPL-licensed jpegxl-rs, so not supported
        !matches!(self, Self::Jxl)
    }
}
```

**Step 2: Define error types**

`crates/slimg-core/src/error.rs`:
```rust
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

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Step 3: Define ImageData and Codec trait**

`crates/slimg-core/src/codec.rs`:
```rust
use crate::error::Result;
use crate::format::Format;

/// Raw decoded image data in RGBA format.
#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA, 4 bytes per pixel
}

impl ImageData {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        debug_assert_eq!(data.len(), (width * height * 4) as usize);
        Self { width, height, data }
    }

    /// Convert to RGB (3 bytes per pixel), dropping alpha.
    pub fn to_rgb(&self) -> Vec<u8> {
        self.data
            .chunks_exact(4)
            .flat_map(|px| [px[0], px[1], px[2]])
            .collect()
    }
}

/// Encoding options common across codecs.
#[derive(Debug, Clone)]
pub struct EncodeOptions {
    pub quality: u8, // 1-100
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self { quality: 80 }
    }
}

/// Trait that all codecs implement.
pub trait Codec {
    fn format(&self) -> Format;
    fn decode(&self, data: &[u8]) -> Result<ImageData>;
    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>>;
}
```

**Step 4: Update lib.rs**

```rust
pub mod codec;
pub mod error;
pub mod format;

pub use codec::{Codec, EncodeOptions, ImageData};
pub use error::{Error, Result};
pub use format::Format;
```

**Step 5: Write tests for Format**

Create `crates/slimg-core/src/format.rs` — add at bottom:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detect_jpeg_from_extension() {
        assert_eq!(Format::from_extension(Path::new("photo.jpg")), Some(Format::Jpeg));
        assert_eq!(Format::from_extension(Path::new("photo.jpeg")), Some(Format::Jpeg));
        assert_eq!(Format::from_extension(Path::new("photo.JPG")), Some(Format::Jpeg));
    }

    #[test]
    fn detect_webp_from_extension() {
        assert_eq!(Format::from_extension(Path::new("image.webp")), Some(Format::WebP));
    }

    #[test]
    fn detect_unknown_extension() {
        assert_eq!(Format::from_extension(Path::new("file.bmp")), None);
        assert_eq!(Format::from_extension(Path::new("noext")), None);
    }

    #[test]
    fn detect_jpeg_from_magic_bytes() {
        let data = [0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Jpeg));
    }

    #[test]
    fn detect_png_from_magic_bytes() {
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Png));
    }

    #[test]
    fn detect_webp_from_magic_bytes() {
        let mut data = vec![0u8; 12];
        data[0..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WEBP");
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::WebP));
    }

    #[test]
    fn detect_qoi_from_magic_bytes() {
        let mut data = vec![0u8; 12];
        data[0..4].copy_from_slice(b"qoif");
        assert_eq!(Format::from_magic_bytes(&data), Some(Format::Qoi));
    }

    #[test]
    fn jxl_cannot_encode() {
        assert!(!Format::Jxl.can_encode());
        assert!(Format::Jpeg.can_encode());
        assert!(Format::WebP.can_encode());
    }
}
```

**Step 6: Run tests**

Run: `cargo test -p slimg-core`
Expected: All tests pass

**Step 7: Commit**

```bash
git add -A
git commit -m "feat: add core types — Format, ImageData, Codec trait, Error"
```

---

## Task 3: JPEG Codec (MozJPEG)

**Files:**
- Create: `crates/slimg-core/src/codec/mod.rs`
- Create: `crates/slimg-core/src/codec/jpeg.rs`
- Modify: `crates/slimg-core/Cargo.toml` — add `mozjpeg = "0.10"`
- Modify: `crates/slimg-core/src/lib.rs` — update module path
- Test: `crates/slimg-core/src/codec/jpeg.rs` (inline tests)

**Step 1: Add dependency**

Add to `crates/slimg-core/Cargo.toml`:
```toml
mozjpeg = "0.10"
```

**Step 2: Create codec module**

`crates/slimg-core/src/codec/mod.rs`:
```rust
pub mod jpeg;

use crate::error::Result;
use crate::format::Format;
use crate::{EncodeOptions, ImageData};

/// Get the appropriate codec for a format.
pub fn get_codec(format: Format) -> Result<Box<dyn Codec>> {
    match format {
        Format::Jpeg => Ok(Box::new(jpeg::JpegCodec)),
        _ => Err(crate::Error::UnsupportedFormat(format)),
    }
}

/// Trait that all codecs implement.
pub trait Codec: Send + Sync {
    fn format(&self) -> Format;
    fn decode(&self, data: &[u8]) -> Result<ImageData>;
    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>>;
}
```

Note: Move the `Codec` trait from `codec.rs` into `codec/mod.rs` and delete the old `codec.rs`. Keep `ImageData` and `EncodeOptions` in a new `types.rs` or directly in `lib.rs`.

**Step 3: Implement JPEG codec**

`crates/slimg-core/src/codec/jpeg.rs`:
```rust
use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::format::Format;
use crate::{EncodeOptions, ImageData};

pub struct JpegCodec;

impl Codec for JpegCodec {
    fn format(&self) -> Format {
        Format::Jpeg
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let result = std::panic::catch_unwind(|| -> Result<ImageData> {
            let d = mozjpeg::Decompress::new_mem(data)
                .map_err(|e| Error::Decode(format!("JPEG decompress init: {e}")))?;
            let width = d.width() as u32;
            let height = d.height() as u32;
            let mut decompress = d
                .rgba()
                .map_err(|e| Error::Decode(format!("JPEG start decompress: {e}")))?;
            let pixels: Vec<[u8; 4]> = decompress
                .read_scanlines()
                .ok_or_else(|| Error::Decode("JPEG: failed to read scanlines".into()))?;
            decompress.finish().map_err(|e| Error::Decode(format!("JPEG finish: {e}")))?;
            let data: Vec<u8> = pixels.into_iter().flat_map(|px| px).collect();
            Ok(ImageData::new(width, height, data))
        });
        result.unwrap_or_else(|_| Err(Error::Decode("JPEG: decoder panicked".into())))
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        let quality = options.quality.clamp(1, 100) as f32;
        let rgb = image.to_rgb();
        let result = std::panic::catch_unwind(|| -> Result<Vec<u8>> {
            let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
            comp.set_size(image.width as usize, image.height as usize);
            comp.set_quality(quality);
            comp.set_progressive_mode();
            comp.set_optimize_scans(true);
            comp.set_optimize_coding(true);
            let mut comp = comp
                .start_compress(Vec::new())
                .map_err(|e| Error::Encode(format!("JPEG start compress: {e}")))?;
            comp.write_scanlines(&rgb)
                .map_err(|e| Error::Encode(format!("JPEG write scanlines: {e}")))?;
            let data = comp
                .finish()
                .map_err(|e| Error::Encode(format!("JPEG finish: {e}")))?;
            Ok(data)
        });
        result.unwrap_or_else(|_| Err(Error::Encode("JPEG: encoder panicked".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let size = (width * height * 4) as usize;
        let mut data = vec![0u8; size];
        // Red/green gradient
        for y in 0..height {
            for x in 0..width {
                let i = ((y * width + x) * 4) as usize;
                data[i] = (x * 255 / width) as u8;     // R
                data[i + 1] = (y * 255 / height) as u8; // G
                data[i + 2] = 128;                       // B
                data[i + 3] = 255;                       // A
            }
        }
        ImageData::new(width, height, data)
    }

    #[test]
    fn encode_and_decode_roundtrip() {
        let codec = JpegCodec;
        let original = create_test_image(64, 64);
        let options = EncodeOptions { quality: 90 };

        let encoded = codec.encode(&original, &options).unwrap();
        assert!(!encoded.is_empty());
        // JPEG magic bytes
        assert_eq!(&encoded[0..3], &[0xFF, 0xD8, 0xFF]);

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.width, 64);
        assert_eq!(decoded.height, 64);
        assert_eq!(decoded.data.len(), 64 * 64 * 4);
    }

    #[test]
    fn encode_produces_smaller_at_lower_quality() {
        let codec = JpegCodec;
        let image = create_test_image(128, 128);

        let high = codec.encode(&image, &EncodeOptions { quality: 95 }).unwrap();
        let low = codec.encode(&image, &EncodeOptions { quality: 30 }).unwrap();
        assert!(low.len() < high.len());
    }

    #[test]
    fn decode_invalid_data_returns_error() {
        let codec = JpegCodec;
        let result = codec.decode(b"not a jpeg");
        assert!(result.is_err());
    }
}
```

**Step 4: Update lib.rs**

```rust
pub mod codec;
pub mod error;
pub mod format;

pub use codec::{Codec, ImageData, EncodeOptions};
pub use error::{Error, Result};
pub use format::Format;
```

Move `ImageData` and `EncodeOptions` into `codec/mod.rs` (they were in the old `codec.rs`).

**Step 5: Run tests**

Run: `cargo test -p slimg-core`
Expected: All tests pass

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: add JPEG codec with MozJPEG encoding/decoding"
```

---

## Task 4: PNG Codec (OxiPNG)

**Files:**
- Create: `crates/slimg-core/src/codec/png.rs`
- Modify: `crates/slimg-core/src/codec/mod.rs` — add `pub mod png;` and update `get_codec`
- Modify: `crates/slimg-core/Cargo.toml` — add `oxipng`

**Step 1: Add dependency**

```toml
oxipng = { version = "10", default-features = false, features = ["parallel", "zopfli"] }
```

**Step 2: Implement PNG codec**

`crates/slimg-core/src/codec/png.rs`:
```rust
use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::format::Format;
use crate::{EncodeOptions, ImageData};

pub struct PngCodec;

impl Codec for PngCodec {
    fn format(&self) -> Format {
        Format::Png
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let img = image::load_from_memory_with_format(data, image::ImageFormat::Png)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(ImageData::new(width, height, rgba.into_raw()))
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        // First, encode as raw PNG
        let mut png_buf: Vec<u8> = Vec::new();
        {
            let encoder = image::codecs::png::PngEncoder::new(&mut png_buf);
            image::ImageEncoder::write_image(
                encoder,
                &image.data,
                image.width,
                image.height,
                image::ExtendedColorType::Rgba8,
            )?;
        }

        // Then optimize with oxipng
        // Map quality 1-100 to oxipng preset 6-0 (inverted: higher quality = less aggressive)
        let preset = match options.quality {
            90..=100 => 1,
            70..=89 => 2,
            50..=69 => 3,
            30..=49 => 4,
            _ => 6,
        };
        let opts = oxipng::Options::from_preset(preset);
        let optimized = oxipng::optimize_from_memory(&png_buf, &opts)
            .map_err(|e| Error::Encode(format!("PNG optimization: {e}")))?;
        Ok(optimized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let size = (width * height * 4) as usize;
        let data = vec![128u8; size];
        ImageData::new(width, height, data)
    }

    #[test]
    fn encode_and_decode_roundtrip() {
        let codec = PngCodec;
        let original = create_test_image(32, 32);
        let options = EncodeOptions { quality: 80 };

        let encoded = codec.encode(&original, &options).unwrap();
        assert!(!encoded.is_empty());
        // PNG magic bytes
        assert_eq!(&encoded[0..4], &[0x89, 0x50, 0x4E, 0x47]);

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.width, 32);
        assert_eq!(decoded.height, 32);
        // PNG is lossless, so data should match
        assert_eq!(decoded.data, original.data);
    }

    #[test]
    fn decode_invalid_data_returns_error() {
        let codec = PngCodec;
        assert!(codec.decode(b"not png").is_err());
    }
}
```

**Step 3: Update codec/mod.rs**

Add `pub mod png;` and update `get_codec`:
```rust
Format::Png => Ok(Box::new(png::PngCodec)),
```

**Step 4: Run tests**

Run: `cargo test -p slimg-core`
Expected: All tests pass

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add PNG codec with OxiPNG optimization"
```

---

## Task 5: WebP Codec

**Files:**
- Create: `crates/slimg-core/src/codec/webp.rs`
- Modify: `crates/slimg-core/src/codec/mod.rs`
- Modify: `crates/slimg-core/Cargo.toml` — add `webp = "0.3"`

**Step 1: Add dependency**

```toml
webp = { version = "0.3", default-features = false }
```

**Step 2: Implement WebP codec**

`crates/slimg-core/src/codec/webp.rs`:
```rust
use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::format::Format;
use crate::{EncodeOptions, ImageData};

pub struct WebPCodec;

impl Codec for WebPCodec {
    fn format(&self) -> Format {
        Format::WebP
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let decoder = webp::Decoder::new(data);
        let image = decoder
            .decode()
            .ok_or_else(|| Error::Decode("WebP: invalid data".into()))?;
        let width = image.width();
        let height = image.height();
        // Convert to RGBA via image crate integration or manual
        let img = image::load_from_memory_with_format(data, image::ImageFormat::WebP)?;
        let rgba = img.to_rgba8();
        Ok(ImageData::new(width, height, rgba.into_raw()))
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        let quality = options.quality.clamp(1, 100) as f32;
        let encoder = webp::Encoder::from_rgba(&image.data, image.width, image.height);
        let mem = encoder.encode(quality);
        Ok(mem.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let size = (width * height * 4) as usize;
        let mut data = vec![0u8; size];
        for i in (0..size).step_by(4) {
            data[i] = 200;     // R
            data[i + 1] = 100; // G
            data[i + 2] = 50;  // B
            data[i + 3] = 255; // A
        }
        ImageData::new(width, height, data)
    }

    #[test]
    fn encode_and_decode_roundtrip() {
        let codec = WebPCodec;
        let original = create_test_image(64, 64);
        let options = EncodeOptions { quality: 90 };

        let encoded = codec.encode(&original, &options).unwrap();
        assert!(!encoded.is_empty());

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.width, 64);
        assert_eq!(decoded.height, 64);
    }

    #[test]
    fn lower_quality_produces_smaller_file() {
        let codec = WebPCodec;
        let image = create_test_image(128, 128);

        let high = codec.encode(&image, &EncodeOptions { quality: 95 }).unwrap();
        let low = codec.encode(&image, &EncodeOptions { quality: 20 }).unwrap();
        assert!(low.len() < high.len());
    }
}
```

**Step 3: Update codec/mod.rs**

Add `pub mod webp;` and:
```rust
Format::WebP => Ok(Box::new(webp::WebPCodec)),
```

**Step 4: Run tests**

Run: `cargo test -p slimg-core`
Expected: All tests pass

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add WebP codec with libwebp encoding/decoding"
```

---

## Task 6: AVIF Codec (ravif)

**Files:**
- Create: `crates/slimg-core/src/codec/avif.rs`
- Modify: `crates/slimg-core/src/codec/mod.rs`
- Modify: `crates/slimg-core/Cargo.toml` — add `ravif`, `rgb`, `imgref`

**Step 1: Add dependencies**

```toml
ravif = "0.13"
rgb = "0.8"
imgref = "1"
```

**Step 2: Implement AVIF codec**

`crates/slimg-core/src/codec/avif.rs`:
```rust
use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::format::Format;
use crate::{EncodeOptions, ImageData};
use imgref::Img;
use rgb::RGBA8;

pub struct AvifCodec;

impl Codec for AvifCodec {
    fn format(&self) -> Format {
        Format::Avif
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        // Use image crate for AVIF decoding
        let img = image::load_from_memory_with_format(data, image::ImageFormat::Avif)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(ImageData::new(width, height, rgba.into_raw()))
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        let quality = options.quality.clamp(1, 100) as f32;
        let pixels: Vec<RGBA8> = image
            .data
            .chunks_exact(4)
            .map(|px| RGBA8::new(px[0], px[1], px[2], px[3]))
            .collect();
        let buffer = Img::new(pixels.as_slice(), image.width as usize, image.height as usize);
        let encoded = ravif::Encoder::new()
            .with_quality(quality)
            .with_speed(6) // reasonable default: 1=slowest/best, 10=fastest
            .encode_rgba(buffer)
            .map_err(|e| Error::Encode(format!("AVIF encode: {e}")))?;
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
                data[i] = (x * 255 / width) as u8;
                data[i + 1] = (y * 255 / height) as u8;
                data[i + 2] = 100;
                data[i + 3] = 255;
            }
        }
        ImageData::new(width, height, data)
    }

    #[test]
    fn encode_produces_valid_avif() {
        let codec = AvifCodec;
        let image = create_test_image(64, 64);
        let encoded = codec.encode(&image, &EncodeOptions { quality: 70 }).unwrap();
        assert!(!encoded.is_empty());
        // AVIF container starts with ftyp box
        assert_eq!(&encoded[4..8], b"ftyp");
    }

    #[test]
    fn encode_and_decode_roundtrip() {
        let codec = AvifCodec;
        let original = create_test_image(64, 64);
        let encoded = codec.encode(&original, &EncodeOptions { quality: 80 }).unwrap();
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.width, 64);
        assert_eq!(decoded.height, 64);
    }
}
```

**Step 3: Update codec/mod.rs**

Add `pub mod avif;` and:
```rust
Format::Avif => Ok(Box::new(avif::AvifCodec)),
```

**Step 4: Run tests**

Run: `cargo test -p slimg-core`
Expected: All tests pass (AVIF encode is slow, tests may take a few seconds)

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add AVIF codec with ravif encoding"
```

---

## Task 7: QOI Codec & JXL Decoder

**Files:**
- Create: `crates/slimg-core/src/codec/qoi.rs`
- Create: `crates/slimg-core/src/codec/jxl.rs`
- Modify: `crates/slimg-core/src/codec/mod.rs`
- Modify: `crates/slimg-core/Cargo.toml` — add `rapid-qoi`, `jxl-oxide`

**Step 1: Add dependencies**

```toml
rapid-qoi = "0.6"
jxl-oxide = "0.12"
```

**Step 2: Implement QOI codec**

`crates/slimg-core/src/codec/qoi.rs`:
```rust
use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::format::Format;
use crate::{EncodeOptions, ImageData};
use rapid_qoi::{Colors, Qoi};

pub struct QoiCodec;

impl Codec for QoiCodec {
    fn format(&self) -> Format {
        Format::Qoi
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let (header, pixels) = Qoi::decode_alloc(data)
            .map_err(|e| Error::Decode(format!("QOI: {e}")))?;
        // QOI decodes as RGBA (4 channels)
        Ok(ImageData::new(header.width, header.height, pixels))
    }

    fn encode(&self, image: &ImageData, _options: &EncodeOptions) -> Result<Vec<u8>> {
        // QOI is lossless, quality option is ignored
        let header = Qoi {
            width: image.width,
            height: image.height,
            colors: Colors::SrgbLinA,
        };
        header
            .encode_alloc(&image.data)
            .map_err(|e| Error::Encode(format!("QOI: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_and_decode_roundtrip() {
        let codec = QoiCodec;
        let original = ImageData::new(16, 16, vec![128u8; 16 * 16 * 4]);
        let encoded = codec.encode(&original, &EncodeOptions::default()).unwrap();
        assert!(encoded.starts_with(b"qoif"));
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.width, 16);
        assert_eq!(decoded.height, 16);
        assert_eq!(decoded.data, original.data); // lossless
    }
}
```

**Step 3: Implement JXL decoder (decode only)**

`crates/slimg-core/src/codec/jxl.rs`:
```rust
use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::format::Format;
use crate::{EncodeOptions, ImageData};

pub struct JxlCodec;

impl Codec for JxlCodec {
    fn format(&self) -> Format {
        Format::Jxl
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        // Use image crate for JXL decoding (delegates to jxl-oxide internally)
        let img = image::load_from_memory(data)
            .map_err(|e| Error::Decode(format!("JXL: {e}")))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(ImageData::new(width, height, rgba.into_raw()))
    }

    fn encode(&self, _image: &ImageData, _options: &EncodeOptions) -> Result<Vec<u8>> {
        Err(Error::EncodingNotSupported(Format::Jxl))
    }
}
```

**Step 4: Update codec/mod.rs**

Add `pub mod qoi;` and `pub mod jxl;`, update `get_codec`:
```rust
Format::Qoi => Ok(Box::new(qoi::QoiCodec)),
Format::Jxl => Ok(Box::new(jxl::JxlCodec)),
```

**Step 5: Run tests**

Run: `cargo test -p slimg-core`
Expected: All tests pass

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: add QOI codec and JXL decoder (encode not supported due to GPL)"
```

---

## Task 8: Resize Module

**Files:**
- Create: `crates/slimg-core/src/resize.rs`
- Modify: `crates/slimg-core/src/lib.rs`

**Step 1: Implement resize**

`crates/slimg-core/src/resize.rs`:
```rust
use crate::error::Result;
use crate::ImageData;
use image::imageops::FilterType;

#[derive(Debug, Clone)]
pub enum ResizeMode {
    /// Set exact width, calculate height to preserve aspect ratio.
    Width(u32),
    /// Set exact height, calculate width to preserve aspect ratio.
    Height(u32),
    /// Set exact dimensions (may distort).
    Exact(u32, u32),
    /// Fit within bounds, preserving aspect ratio.
    Fit(u32, u32),
    /// Scale by factor (e.g., 0.5 = half size).
    Scale(f64),
}

pub fn resize(image: &ImageData, mode: &ResizeMode) -> Result<ImageData> {
    let (target_w, target_h) = calculate_dimensions(image.width, image.height, mode);
    if target_w == 0 || target_h == 0 {
        return Err(crate::Error::Resize("target dimensions must be > 0".into()));
    }

    let img_buf = image::RgbaImage::from_raw(image.width, image.height, image.data.clone())
        .ok_or_else(|| crate::Error::Resize("invalid image data".into()))?;
    let dyn_img = image::DynamicImage::ImageRgba8(img_buf);

    let resized = match mode {
        ResizeMode::Exact(_, _) => dyn_img.resize_exact(target_w, target_h, FilterType::Lanczos3),
        _ => dyn_img.resize(target_w, target_h, FilterType::Lanczos3),
    };

    let rgba = resized.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(ImageData::new(w, h, rgba.into_raw()))
}

fn calculate_dimensions(orig_w: u32, orig_h: u32, mode: &ResizeMode) -> (u32, u32) {
    match mode {
        ResizeMode::Width(w) => {
            let ratio = *w as f64 / orig_w as f64;
            (*w, (orig_h as f64 * ratio).round() as u32)
        }
        ResizeMode::Height(h) => {
            let ratio = *h as f64 / orig_h as f64;
            ((orig_w as f64 * ratio).round() as u32, *h)
        }
        ResizeMode::Exact(w, h) => (*w, *h),
        ResizeMode::Fit(w, h) => {
            let ratio_w = *w as f64 / orig_w as f64;
            let ratio_h = *h as f64 / orig_h as f64;
            let ratio = ratio_w.min(ratio_h);
            (
                (orig_w as f64 * ratio).round() as u32,
                (orig_h as f64 * ratio).round() as u32,
            )
        }
        ResizeMode::Scale(s) => (
            (orig_w as f64 * s).round() as u32,
            (orig_h as f64 * s).round() as u32,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_image(w: u32, h: u32) -> ImageData {
        ImageData::new(w, h, vec![128u8; (w * h * 4) as usize])
    }

    #[test]
    fn resize_by_width_preserves_ratio() {
        let img = test_image(200, 100);
        let result = resize(&img, &ResizeMode::Width(100)).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn resize_by_height_preserves_ratio() {
        let img = test_image(200, 100);
        let result = resize(&img, &ResizeMode::Height(50)).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn resize_fit_within_bounds() {
        let img = test_image(400, 200);
        let result = resize(&img, &ResizeMode::Fit(100, 100)).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn resize_by_scale() {
        let img = test_image(200, 100);
        let result = resize(&img, &ResizeMode::Scale(0.5)).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn resize_exact_ignores_ratio() {
        let img = test_image(200, 100);
        let result = resize(&img, &ResizeMode::Exact(50, 50)).unwrap();
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);
    }
}
```

**Step 2: Export from lib.rs**

```rust
pub mod resize;
pub use resize::ResizeMode;
```

**Step 3: Run tests**

Run: `cargo test -p slimg-core`
Expected: All tests pass

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: add resize module with multiple resize modes"
```

---

## Task 9: Pipeline (High-Level API)

**Files:**
- Create: `crates/slimg-core/src/pipeline.rs`
- Modify: `crates/slimg-core/src/lib.rs`

**Step 1: Implement pipeline**

`crates/slimg-core/src/pipeline.rs`:
```rust
use std::path::Path;

use crate::codec::get_codec;
use crate::error::{Error, Result};
use crate::format::Format;
use crate::resize::{self, ResizeMode};
use crate::{EncodeOptions, ImageData};

/// Options for the convert/optimize pipeline.
#[derive(Debug, Clone)]
pub struct PipelineOptions {
    pub format: Format,
    pub quality: u8,
    pub resize: Option<ResizeMode>,
}

/// Result of a pipeline operation.
pub struct PipelineResult {
    pub data: Vec<u8>,
    pub format: Format,
}

impl PipelineResult {
    pub fn save(&self, path: &Path) -> Result<()> {
        std::fs::write(path, &self.data)?;
        Ok(())
    }
}

/// Decode an image from raw bytes, detecting format from magic bytes.
pub fn decode(data: &[u8]) -> Result<(ImageData, Format)> {
    let format = Format::from_magic_bytes(data)
        .ok_or_else(|| Error::UnknownFormat("unrecognized image format".into()))?;
    let codec = get_codec(format)?;
    let image = codec.decode(data)?;
    Ok((image, format))
}

/// Decode an image from a file path.
pub fn decode_file(path: &Path) -> Result<(ImageData, Format)> {
    let data = std::fs::read(path)?;
    decode(&data)
}

/// Convert an image: decode → optional resize → encode to target format.
pub fn convert(image: &ImageData, options: &PipelineOptions) -> Result<PipelineResult> {
    if !options.format.can_encode() {
        return Err(Error::EncodingNotSupported(options.format));
    }

    let image = match &options.resize {
        Some(mode) => resize::resize(image, mode)?,
        None => image.clone(),
    };

    let codec = get_codec(options.format)?;
    let encode_opts = EncodeOptions {
        quality: options.quality,
    };
    let data = codec.encode(&image, &encode_opts)?;
    Ok(PipelineResult {
        data,
        format: options.format,
    })
}

/// Optimize an image in-place (same format, better compression).
pub fn optimize(data: &[u8], quality: u8) -> Result<PipelineResult> {
    let (image, format) = decode(data)?;
    convert(&image, &PipelineOptions {
        format,
        quality,
        resize: None,
    })
}

/// Generate the output path based on input path and target format.
pub fn output_path(input: &Path, format: Format, output: Option<&Path>) -> std::path::PathBuf {
    match output {
        Some(p) if p.is_dir() => {
            let stem = input.file_stem().unwrap_or_default();
            p.join(stem).with_extension(format.extension())
        }
        Some(p) => p.to_path_buf(),
        None => input.with_extension(format.extension()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_path_changes_extension() {
        let result = output_path(Path::new("/tmp/photo.jpg"), Format::WebP, None);
        assert_eq!(result, Path::new("/tmp/photo.webp"));
    }

    #[test]
    fn output_path_with_explicit_output() {
        let result = output_path(
            Path::new("/tmp/photo.jpg"),
            Format::Avif,
            Some(Path::new("/out/result.avif")),
        );
        assert_eq!(result, Path::new("/out/result.avif"));
    }

    #[test]
    fn jxl_encode_returns_error() {
        let image = ImageData::new(1, 1, vec![0, 0, 0, 255]);
        let result = convert(&image, &PipelineOptions {
            format: Format::Jxl,
            quality: 80,
            resize: None,
        });
        assert!(result.is_err());
    }
}
```

**Step 2: Export from lib.rs**

```rust
pub mod pipeline;
pub use pipeline::{PipelineOptions, PipelineResult, convert, decode, decode_file, optimize, output_path};
```

**Step 3: Run tests**

Run: `cargo test -p slimg-core`
Expected: All tests pass

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: add pipeline module — high-level convert/optimize/decode API"
```

---

## Task 10: CLI — Subcommands

**Files:**
- Modify: `cli/src/main.rs`
- Create: `cli/src/commands/mod.rs`
- Create: `cli/src/commands/convert.rs`
- Create: `cli/src/commands/optimize.rs`
- Create: `cli/src/commands/resize.rs`

**Step 1: Define CLI structure**

`cli/src/main.rs`:
```rust
mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "slimg", version, about = "Image optimization CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert image to a different format with compression
    Convert(commands::convert::ConvertArgs),
    /// Optimize image in the same format (reduce file size)
    Optimize(commands::optimize::OptimizeArgs),
    /// Resize image with optional format conversion
    Resize(commands::resize::ResizeArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Convert(args) => commands::convert::run(args),
        Commands::Optimize(args) => commands::optimize::run(args),
        Commands::Resize(args) => commands::resize::run(args),
    }
}
```

**Step 2: Implement convert command**

`cli/src/commands/mod.rs`:
```rust
pub mod convert;
pub mod optimize;
pub mod resize;

use clap::ValueEnum;
use slimg_core::Format;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormatArg {
    Jpeg,
    Png,
    Webp,
    Avif,
    Qoi,
}

impl From<FormatArg> for Format {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::Jpeg => Format::Jpeg,
            FormatArg::Png => Format::Png,
            FormatArg::Webp => Format::WebP,
            FormatArg::Avif => Format::Avif,
            FormatArg::Qoi => Format::Qoi,
        }
    }
}
```

`cli/src/commands/convert.rs`:
```rust
use std::path::PathBuf;

use clap::Args;
use slimg_core::{Format, PipelineOptions, decode_file, convert, output_path};

use super::FormatArg;

#[derive(Args)]
pub struct ConvertArgs {
    /// Input file or directory
    input: PathBuf,

    /// Output format
    #[arg(short, long)]
    format: FormatArg,

    /// Quality (1-100)
    #[arg(short, long, default_value_t = 80)]
    quality: u8,

    /// Output path (file or directory)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Process subdirectories
    #[arg(long)]
    recursive: bool,
}

pub fn run(args: ConvertArgs) -> anyhow::Result<()> {
    let target_format: Format = args.format.into();
    let files = collect_files(&args.input, args.recursive)?;

    for file in &files {
        let (image, _source_format) = decode_file(file)?;
        let result = convert(&image, &PipelineOptions {
            format: target_format,
            quality: args.quality,
            resize: None,
        })?;
        let out = output_path(file, target_format, args.output.as_deref());
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        result.save(&out)?;
        let original_size = std::fs::metadata(file)?.len();
        let new_size = result.data.len() as u64;
        let ratio = if original_size > 0 {
            (new_size as f64 / original_size as f64 * 100.0) as u32
        } else {
            100
        };
        eprintln!(
            "{} → {} ({} → {} bytes, {}%)",
            file.display(),
            out.display(),
            original_size,
            new_size,
            ratio,
        );
    }
    Ok(())
}

fn collect_files(path: &PathBuf, recursive: bool) -> anyhow::Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.clone()]);
    }
    if !path.is_dir() {
        anyhow::bail!("'{}' is not a file or directory", path.display());
    }

    let mut files = Vec::new();
    let entries = if recursive {
        walkdir(path)?
    } else {
        std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect()
    };

    for entry in entries {
        if entry.is_file() {
            if let Some(ext) = entry.extension() {
                let ext = ext.to_string_lossy().to_ascii_lowercase();
                if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "avif" | "jxl" | "qoi") {
                    files.push(entry);
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

fn walkdir(path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            result.extend(walkdir(&path)?);
        } else {
            result.push(path);
        }
    }
    Ok(result)
}
```

**Step 3: Implement optimize command**

`cli/src/commands/optimize.rs`:
```rust
use std::path::PathBuf;

use clap::Args;
use slimg_core::{PipelineOptions, decode_file, convert, output_path};

#[derive(Args)]
pub struct OptimizeArgs {
    /// Input file or directory
    input: PathBuf,

    /// Quality (1-100)
    #[arg(short, long, default_value_t = 80)]
    quality: u8,

    /// Output path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Overwrite original files
    #[arg(long)]
    overwrite: bool,

    /// Process subdirectories
    #[arg(long)]
    recursive: bool,
}

pub fn run(args: OptimizeArgs) -> anyhow::Result<()> {
    let files = super::convert::collect_files(&args.input, args.recursive)?;

    for file in &files {
        let (image, format) = decode_file(file)?;
        if !format.can_encode() {
            eprintln!("skipping {} (encoding not supported for {:?})", file.display(), format);
            continue;
        }
        let result = convert(&image, &PipelineOptions {
            format,
            quality: args.quality,
            resize: None,
        })?;

        let out = if args.overwrite {
            file.clone()
        } else {
            output_path(file, format, args.output.as_deref())
        };

        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let original_size = std::fs::metadata(file)?.len();
        let new_size = result.data.len() as u64;

        // Only write if smaller (or overwrite forced)
        if new_size < original_size || args.overwrite {
            result.save(&out)?;
            let saved = original_size.saturating_sub(new_size);
            eprintln!(
                "{} optimized: {} → {} bytes (saved {} bytes)",
                file.display(),
                original_size,
                new_size,
                saved,
            );
        } else {
            eprintln!(
                "{} skipped: already optimal ({} bytes)",
                file.display(),
                original_size,
            );
        }
    }
    Ok(())
}
```

Note: Make `collect_files` in `convert.rs` `pub(crate)` so `optimize.rs` and `resize.rs` can use it.

**Step 4: Implement resize command**

`cli/src/commands/resize.rs`:
```rust
use std::path::PathBuf;

use clap::Args;
use slimg_core::{Format, PipelineOptions, ResizeMode, decode_file, convert, output_path};

use super::FormatArg;

#[derive(Args)]
pub struct ResizeArgs {
    /// Input file
    input: PathBuf,

    /// Target width
    #[arg(long)]
    width: Option<u32>,

    /// Target height
    #[arg(long)]
    height: Option<u32>,

    /// Scale factor (e.g., 0.5)
    #[arg(long)]
    scale: Option<f64>,

    /// Output format (defaults to input format)
    #[arg(short, long)]
    format: Option<FormatArg>,

    /// Quality (1-100)
    #[arg(short, long, default_value_t = 80)]
    quality: u8,

    /// Output path
    #[arg(short, long)]
    output: Option<PathBuf>,
}

pub fn run(args: ResizeArgs) -> anyhow::Result<()> {
    let resize_mode = match (args.width, args.height, args.scale) {
        (Some(w), Some(h), _) => ResizeMode::Fit(w, h),
        (Some(w), None, _) => ResizeMode::Width(w),
        (None, Some(h), _) => ResizeMode::Height(h),
        (_, _, Some(s)) => ResizeMode::Scale(s),
        _ => anyhow::bail!("specify --width, --height, or --scale"),
    };

    let (image, source_format) = decode_file(&args.input)?;
    let target_format = args.format.map(Format::from).unwrap_or(source_format);

    let result = convert(&image, &PipelineOptions {
        format: target_format,
        quality: args.quality,
        resize: Some(resize_mode),
    })?;

    let out = output_path(&args.input, target_format, args.output.as_deref());
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    result.save(&out)?;
    eprintln!("{} → {} ({}x{})", args.input.display(), out.display(), image.width, image.height);
    Ok(())
}
```

**Step 5: Build and test CLI**

Run: `cargo build -p slimg`
Expected: Compiles successfully

Run: `cargo run -p slimg -- --help`
Expected: Shows subcommands (convert, optimize, resize)

Run: `cargo run -p slimg -- convert --help`
Expected: Shows convert options

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: add CLI with convert, optimize, resize subcommands"
```

---

## Task 11: Integration Test with Real Images

**Files:**
- Create: `tests/fixtures/` — add a small test JPEG and PNG
- Create: `tests/integration_test.rs` (workspace level) or `cli/tests/`

**Step 1: Generate test fixtures programmatically**

Create `crates/slimg-core/tests/integration.rs`:
```rust
use slimg_core::*;

fn create_test_image() -> ImageData {
    let (w, h) = (100, 80);
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            data[i] = (x * 255 / w) as u8;
            data[i + 1] = (y * 255 / h) as u8;
            data[i + 2] = 128;
            data[i + 3] = 255;
        }
    }
    ImageData::new(w, h, data)
}

#[test]
fn convert_jpeg_to_webp() {
    let image = create_test_image();
    let jpeg = convert(&image, &PipelineOptions {
        format: Format::Jpeg,
        quality: 90,
        resize: None,
    }).unwrap();
    let (decoded, _) = decode(&jpeg.data).unwrap();
    let webp = convert(&decoded, &PipelineOptions {
        format: Format::WebP,
        quality: 80,
        resize: None,
    }).unwrap();
    assert!(!webp.data.is_empty());
    assert_eq!(webp.format, Format::WebP);
}

#[test]
fn convert_with_resize() {
    let image = create_test_image();
    let result = convert(&image, &PipelineOptions {
        format: Format::Png,
        quality: 80,
        resize: Some(ResizeMode::Width(50)),
    }).unwrap();
    let (decoded, _) = decode(&result.data).unwrap();
    assert_eq!(decoded.width, 50);
    assert_eq!(decoded.height, 40); // preserves 100:80 ratio
}

#[test]
fn roundtrip_all_encodable_formats() {
    let image = create_test_image();
    let formats = [Format::Jpeg, Format::Png, Format::WebP, Format::Avif, Format::Qoi];
    for format in formats {
        let encoded = convert(&image, &PipelineOptions {
            format,
            quality: 80,
            resize: None,
        }).unwrap();
        assert!(!encoded.data.is_empty(), "empty output for {format:?}");
        let (decoded, detected) = decode(&encoded.data).unwrap();
        assert_eq!(detected, format);
        assert_eq!(decoded.width, 100);
        assert_eq!(decoded.height, 80);
    }
}
```

**Step 2: Run integration tests**

Run: `cargo test -p slimg-core --test integration`
Expected: All tests pass

**Step 3: Commit**

```bash
git add -A
git commit -m "test: add integration tests for cross-format conversion"
```

---

## Task 12: README & Final Polish

**Files:**
- Create: `README.md`
- Verify: `cargo clippy`, `cargo fmt`

**Step 1: Run clippy and fmt**

Run: `cargo fmt --all`
Run: `cargo clippy --all-targets`
Fix any warnings.

**Step 2: Create README.md**

Write a README covering: what slimg is, installation, usage examples, supported formats, build requirements, license.

**Step 3: Final full test run**

Run: `cargo test --workspace`
Expected: All tests pass

**Step 4: Commit**

```bash
git add -A
git commit -m "docs: add README and polish codebase"
```
