# slimg-core

Image optimization library for Rust. Decode, encode, convert, and resize images using best-in-class codecs.

## Codecs

| Format | Decode | Encode | Encoder |
|--------|--------|--------|---------|
| JPEG | Yes | Yes | MozJPEG |
| PNG | Yes | Yes | OxiPNG (Zopfli) |
| WebP | Yes | Yes | libwebp |
| AVIF | macOS only | Yes | ravif (AV1) |
| QOI | Yes | Yes | rapid-qoi |
| JPEG XL | Yes | No | Decode only (GPL restriction) |

## Usage

```rust
use slimg_core::*;
use std::path::Path;

// Decode from file
let (image, format) = decode_file(Path::new("photo.jpg"))?;

// Convert to WebP at quality 80
let result = convert(&image, &PipelineOptions {
    format: Format::WebP,
    quality: 80,
    resize: None,
})?;
result.save(Path::new("photo.webp"))?;

// Convert and resize in one step
let result = convert(&image, &PipelineOptions {
    format: Format::Avif,
    quality: 60,
    resize: Some(ResizeMode::Width(800)),
})?;

// Optimize in-place (re-encode same format)
let data = std::fs::read("photo.jpg")?;
let optimized = optimize(&data, 75)?;
optimized.save(Path::new("photo.jpg"))?;
```

## Resize Modes

| Mode | Description |
|------|-------------|
| `Width(u32)` | Set width, preserve aspect ratio |
| `Height(u32)` | Set height, preserve aspect ratio |
| `Fit(u32, u32)` | Fit within bounds, preserve aspect ratio |
| `Exact(u32, u32)` | Exact dimensions (may distort) |
| `Scale(f64)` | Scale factor (e.g. 0.5 = half size) |

## CLI

For batch processing and command-line usage, see [slimg](https://crates.io/crates/slimg).

## License

MIT
