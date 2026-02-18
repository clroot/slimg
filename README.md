# slimg

A fast image optimization CLI. Convert, compress, resize, crop, and extend images using modern codecs.

[한국어](./README.ko.md)

## Supported Formats

| Format | Decode | Encode | Notes |
|--------|--------|--------|-------|
| JPEG   | Yes    | Yes    | MozJPEG encoder for superior compression |
| PNG    | Yes    | Yes    | OxiPNG optimizer with Zopfli compression |
| WebP   | Yes    | Yes    | Lossy encoding via libwebp |
| AVIF   | Yes    | Yes    | ravif encoder; dav1d decoder (statically linked) |
| QOI    | Yes    | Yes    | Lossless, fast encode/decode |
| JPEG XL| Yes    | No     | Decode-only (GPL license restriction) |

## Installation

### Cargo (crates.io)

```
cargo install slimg
```

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
- meson + ninja (for building dav1d AVIF decoder from source)
- Set `SYSTEM_DEPS_DAV1D_BUILD_INTERNAL=always` to build dav1d from source

## Usage

For the full usage guide, see [docs/usage.md](./docs/usage.md).

```bash
# Convert format
slimg convert photo.jpg --format webp

# Optimize (re-encode in same format)
slimg optimize photo.jpg --quality 70

# Resize
slimg resize photo.jpg --width 800

# Crop by coordinates
slimg crop photo.jpg --region 100,50,800,600

# Crop to aspect ratio (center-anchored)
slimg crop photo.jpg --aspect 16:9

# Extend to square with padding
slimg extend photo.jpg --aspect 1:1

# Extend with transparent background
slimg extend photo.png --aspect 1:1 --transparent

# Batch processing with format conversion
slimg convert ./images --format webp --output ./output --recursive --jobs 4
```

## Benchmarks

See [docs/benchmarks.md](./docs/benchmarks.md) for detailed performance measurements across all codecs and pipeline operations.

## Language Bindings

| Language | Package | Platforms |
|----------|---------|-----------|
| [Kotlin/JVM](./bindings/kotlin/) | `io.clroot.slimg:slimg-kotlin` | macOS, Linux, Windows |
| [Python](./bindings/python/) | `slimg` | macOS, Linux, Windows |

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
    crop: None,
    extend: None,
    fill_color: None,
})?;

// Save the result
result.save(Path::new("photo.webp"))?;
```

## License

MIT
