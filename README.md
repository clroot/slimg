# slimg

A fast image optimization CLI. Convert, compress, and resize images using modern codecs.

[한국어](./README.ko.md)

## Supported Formats

| Format | Decode | Encode | Notes |
|--------|--------|--------|-------|
| JPEG   | Yes    | Yes    | MozJPEG encoder for superior compression |
| PNG    | Yes    | Yes    | OxiPNG optimizer with Zopfli compression |
| WebP   | Yes    | Yes    | Lossy encoding via libwebp |
| AVIF   | macOS only | Yes | ravif encoder; decoding requires dav1d (macOS via Homebrew) |
| QOI    | Yes    | Yes    | Lossless, fast encode/decode |
| JPEG XL| Yes    | No     | Decode-only (GPL license restriction) |

## Installation

### Homebrew (macOS / Linux)

```
brew install clroot/tap/slimg
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/clroot/slimg/releases/latest):

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `slimg-aarch64-apple-darwin.tar.xz` |
| macOS (Intel) | `slimg-x86_64-apple-darwin.tar.xz` |
| Linux (x86_64) | `slimg-x86_64-unknown-linux-gnu.tar.xz` |
| Linux (ARM64) | `slimg-aarch64-unknown-linux-gnu.tar.xz` |
| Windows (x86_64) | `slimg-x86_64-pc-windows-msvc.zip` |

### From source

```
git clone https://github.com/clroot/slimg.git
cd slimg
cargo install --path cli
```

#### Build requirements

- Rust 1.85+ (edition 2024)
- C compiler (cc)
- nasm (for MozJPEG / rav1e assembly optimizations)
- dav1d (macOS only, for AVIF decoding)

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

# Limit to 4 parallel jobs
slimg convert ./images --format webp --recursive --jobs 4
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

# Limit to 2 parallel jobs (useful for large images)
slimg optimize ./images --recursive --jobs 2
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

## Batch Processing

When processing directories with `--recursive`, slimg uses all available CPU cores via [rayon](https://github.com/rayon-rs/rayon). Use `--jobs` to limit parallelism (useful for large images or memory-constrained environments).

```
# Use 4 threads instead of all cores
slimg convert ./images --format webp --recursive --jobs 4
```

**Error handling** — If a file fails to process, slimg skips it and continues with the rest. A summary of failed files is printed at the end.

**Safe overwrite** — When using `--overwrite`, slimg writes to a temporary file first and renames it on success. If encoding fails, the original file is preserved.

## Benchmarks

See [docs/benchmarks.md](./docs/benchmarks.md) for detailed performance measurements across all codecs and pipeline operations.

## Language Bindings

| Language | Package | Platforms |
|----------|---------|-----------|
| [Kotlin/JVM](./bindings/kotlin/) | `io.clroot.slimg:slimg-kotlin` | macOS, Linux, Windows |

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
