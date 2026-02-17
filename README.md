# slimg

A fast image optimization CLI. Convert, compress, and resize images using modern codecs.

## Supported Formats

| Format | Decode | Encode | Notes |
|--------|--------|--------|-------|
| JPEG   | Yes    | Yes    | MozJPEG encoder for superior compression |
| PNG    | Yes    | Yes    | OxiPNG optimizer with Zopfli compression |
| WebP   | Yes    | Yes    | Lossy encoding via libwebp |
| AVIF   | Yes    | Yes    | ravif encoder (AV1-based) |
| QOI    | Yes    | Yes    | Lossless, fast encode/decode |
| JPEG XL| Yes    | No     | Decode-only (GPL license restriction) |

## Installation

### From source

```
git clone https://github.com/clroot/slimg.git
cd slimg
cargo install --path cli
```

### Build requirements

- Rust 1.85+ (edition 2024)
- C compiler (cc)
- nasm (for MozJPEG assembly optimizations)
- CMake (for native codec builds)

## Usage

### convert

Convert an image to a different format.

```
# Convert a JPEG to WebP at quality 80 (default)
slimg convert photo.jpg --format webp

# Convert to AVIF at quality 60
slimg convert photo.png --format avif --quality 60

# Convert all images in a directory
slimg convert ./images --format webp --output ./output --recursive
```

### optimize

Re-encode an image in the same format to reduce file size.

```
# Optimize a JPEG at quality 80
slimg optimize photo.jpg

# Optimize in-place (overwrite original)
slimg optimize photo.jpg --overwrite

# Optimize a directory of images
slimg optimize ./images --quality 70 --recursive
```

### resize

Resize an image with optional format conversion.

```
# Resize by width (preserves aspect ratio)
slimg resize photo.jpg --width 800

# Resize by height
slimg resize photo.jpg --height 600

# Fit within bounds (preserves aspect ratio)
slimg resize photo.jpg --width 800 --height 600

# Scale by factor
slimg resize photo.jpg --scale 0.5

# Resize and convert format
slimg resize photo.jpg --width 400 --format webp --output thumb.webp
```

## Library

The core functionality is available as a library crate (`slimg-core`):

```rust
use slimg_core::*;

// Decode an image file
let (image, format) = decode_file(Path::new("photo.jpg"))?;

// Convert to WebP
let result = convert(&image, &PipelineOptions {
    format: Format::WebP,
    quality: 80,
    resize: None,
})?;

// Save the result
result.save(Path::new("photo.webp"))?;
```

## License

MIT OR Apache-2.0
