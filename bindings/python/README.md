# slimg

Python bindings for the [slimg](https://github.com/clroot/slimg) image optimization library.

Supports macOS (Apple Silicon, Intel), Linux (x86_64, ARM64), and Windows (x86_64) -- native extensions are bundled in pre-built wheels.

## Installation

```
pip install slimg
```

## Usage

```python
import slimg

# Open an image file
image = slimg.open("photo.jpg")
print(f"{image.width}x{image.height} {image.format}")

# Convert to WebP
result = slimg.convert(image, format="webp", quality=80)
result.save("photo.webp")

# Optimize in the same format
result = slimg.optimize_file("photo.jpg", quality=75)
result.save("optimized.jpg")

# Resize by width (preserves aspect ratio)
resized = slimg.resize(image, width=800)
result = slimg.convert(resized, format="png")
result.save("thumbnail.png")

# Crop to aspect ratio (centre-anchored)
cropped = slimg.crop(image, aspect_ratio=(16, 9))

# Crop by pixel region
cropped = slimg.crop(image, region=(100, 50, 800, 600))

# Extend (pad) to aspect ratio with a fill colour
extended = slimg.extend(image, aspect_ratio=(1, 1), fill=(255, 255, 255))

# Extend with transparent padding (default)
extended = slimg.extend(image, aspect_ratio=(1, 1))
```

## Supported Formats

| Format   | Decode | Encode | Notes |
|----------|--------|--------|-------|
| JPEG     | Yes    | Yes    | MozJPEG encoder |
| PNG      | Yes    | Yes    | OxiPNG + Zopfli compression |
| WebP     | Yes    | Yes    | Lossy encoding via libwebp |
| AVIF     | Yes    | Yes    | ravif encoder; dav1d decoder |
| QOI      | Yes    | Yes    | Lossless, fast encode/decode |
| JPEG XL  | Yes    | No     | Decode-only |

## API Reference

### Functions

| Function | Description |
|----------|-------------|
| `open(path)` | Decode an image file from disk |
| `decode(data)` | Decode image bytes (auto-detects format) |
| `convert(image, format, quality=80)` | Encode image in a target format |
| `resize(image, *, width/height/exact/fit/scale)` | Resize an image |
| `crop(image, *, region/aspect_ratio)` | Crop an image |
| `extend(image, *, aspect_ratio/size, fill)` | Pad an image canvas |
| `optimize(data, quality=80)` | Re-encode bytes to reduce file size |
| `optimize_file(path, quality=80)` | Read a file and re-encode |

### Types

| Type | Description |
|------|-------------|
| `Format` | `JPEG`, `PNG`, `WEBP`, `AVIF`, `JXL`, `QOI` |
| `Image` | Decoded image with `width`, `height`, `data`, `format` |
| `Result` | Encoded output with `data`, `format`, and `save(path)` |
| `Resize` | Factory: `width`, `height`, `exact`, `fit`, `scale` |
| `Crop` | Factory: `region`, `aspect_ratio` |
| `Extend` | Factory: `aspect_ratio`, `size` |
| `SlimgError` | Error with subclasses: `Decode`, `Encode`, `Resize`, `Io`, `Image` |

## Supported Platforms

| Platform | Architecture | Status |
|----------|-------------|--------|
| macOS | Apple Silicon (aarch64) | Supported |
| macOS | Intel (x86_64) | Supported |
| Linux | x86_64 | Supported |
| Linux | ARM64 (aarch64) | Supported |
| Windows | x86_64 | Supported |

## Requirements

- Python 3.9+

## License

MIT OR Apache-2.0
