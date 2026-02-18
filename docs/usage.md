# Usage Guide

[한국어](./usage.ko.md)

slimg provides five commands: **convert**, **optimize**, **resize**, **crop**, and **extend**.

## convert

Convert an image to a different format.

```
slimg convert photo.jpg --format webp
```

| Option | Description |
|--------|-------------|
| `--format`, `-f` | Target format: `jpeg`, `png`, `webp`, `avif`, `qoi` |
| `--quality`, `-q` | Encoding quality 0-100 (default: 80) |
| `--output`, `-o` | Output path (file or directory) |
| `--recursive` | Process subdirectories |
| `--jobs`, `-j` | Number of parallel jobs (default: all cores) |
| `--overwrite` | Overwrite existing files |

**Examples:**

```bash
# Convert a JPEG to WebP at quality 80 (default)
slimg convert photo.jpg --format webp

# Convert to AVIF at quality 60
slimg convert photo.png --format avif --quality 60

# Convert all images in a directory
slimg convert ./images --format webp --output ./output --recursive

# Limit to 4 parallel jobs
slimg convert ./images --format webp --recursive --jobs 4
```

## optimize

Re-encode an image in the same format to reduce file size.

```
slimg optimize photo.jpg
```

| Option | Description |
|--------|-------------|
| `--quality`, `-q` | Encoding quality 0-100 (default: 80) |
| `--output`, `-o` | Output path (file or directory) |
| `--recursive` | Process subdirectories |
| `--jobs`, `-j` | Number of parallel jobs (default: all cores) |
| `--overwrite` | Overwrite original files |

**Examples:**

```bash
# Optimize a JPEG at quality 80
slimg optimize photo.jpg

# Optimize in-place (overwrite original)
slimg optimize photo.jpg --overwrite

# Optimize a directory of images
slimg optimize ./images --quality 70 --recursive

# Limit to 2 parallel jobs (useful for large images)
slimg optimize ./images --recursive --jobs 2
```

## resize

Resize an image with optional format conversion.

```
slimg resize photo.jpg --width 800
```

| Option | Description |
|--------|-------------|
| `--width` | Target width in pixels |
| `--height` | Target height in pixels |
| `--scale` | Scale factor (e.g. `0.5` for half size) |
| `--format`, `-f` | Convert to a different format |
| `--quality`, `-q` | Encoding quality 0-100 (default: 80) |
| `--output`, `-o` | Output path (file or directory) |
| `--recursive` | Process subdirectories |
| `--jobs`, `-j` | Number of parallel jobs (default: all cores) |
| `--overwrite` | Overwrite existing files |

When both `--width` and `--height` are specified, the image is scaled to fit within the bounds while preserving aspect ratio.

**Examples:**

```bash
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

## crop

Crop an image by coordinates or aspect ratio, with optional format conversion.

```
slimg crop photo.jpg --region 100,50,800,600
```

| Option | Description |
|--------|-------------|
| `--region` | Crop region: `x,y,width,height` (e.g. `100,50,800,600`) |
| `--aspect` | Crop to aspect ratio: `width:height` (e.g. `16:9`, `1:1`), center-anchored |
| `--format`, `-f` | Convert to a different format |
| `--quality`, `-q` | Encoding quality 0-100 (default: 80) |
| `--output`, `-o` | Output path (file or directory) |
| `--recursive` | Process subdirectories |
| `--jobs`, `-j` | Number of parallel jobs (default: all cores) |
| `--overwrite` | Overwrite existing files |

`--region` and `--aspect` are mutually exclusive. One of them is required.

**Examples:**

```bash
# Crop by coordinates (x=100, y=50, width=800, height=600)
slimg crop photo.jpg --region 100,50,800,600

# Crop to 16:9 aspect ratio (center-anchored)
slimg crop photo.jpg --aspect 16:9

# Crop to square (1:1)
slimg crop photo.jpg --aspect 1:1

# Crop and convert to WebP
slimg crop photo.jpg --region 0,0,500,500 --format webp

# Batch crop all images in a directory
slimg crop ./images --aspect 16:9 --output ./cropped --recursive
```

## extend

Extend an image by adding padding to match a target aspect ratio or size. The original image is centered on the new canvas.

```
slimg extend photo.jpg --aspect 1:1
```

| Option | Description |
|--------|-------------|
| `--aspect` | Target aspect ratio: `width:height` (e.g. `1:1`, `16:9`) |
| `--size` | Target canvas size: `WIDTHxHEIGHT` (e.g. `1920x1080`) |
| `--color` | Fill color as hex (e.g. `'#FF0000'`, `'000000'`). Default: white |
| `--transparent` | Use transparent background (for PNG, WebP, etc.) |
| `--format`, `-f` | Convert to a different format |
| `--quality`, `-q` | Encoding quality 0-100 (default: 80) |
| `--output`, `-o` | Output path (file or directory) |
| `--recursive` | Process subdirectories |
| `--jobs`, `-j` | Number of parallel jobs (default: all cores) |
| `--overwrite` | Overwrite existing files |

`--aspect` and `--size` are mutually exclusive. One of them is required.
`--color` and `--transparent` are mutually exclusive.

**Examples:**

```bash
# Extend to square (1:1) with white padding
slimg extend photo.jpg --aspect 1:1

# Extend to 16:9 with black padding
slimg extend photo.jpg --aspect 16:9 --color '#000000'

# Extend to exact size with transparent background (PNG)
slimg extend photo.png --size 1920x1080 --transparent

# Extend and convert format
slimg extend photo.jpg --aspect 1:1 --transparent --format png

# Batch extend all images in a directory
slimg extend ./images --aspect 1:1 --output ./squared --recursive
```

## Batch Processing

When processing directories with `--recursive`, slimg uses all available CPU cores via [rayon](https://github.com/rayon-rs/rayon). Use `--jobs` to limit parallelism.

```bash
# Use 4 threads instead of all cores
slimg convert ./images --format webp --recursive --jobs 4
```

**Error handling** — If a file fails to process, slimg skips it and continues. A summary of failed files is printed at the end.

**Safe overwrite** — When using `--overwrite`, slimg writes to a temporary file first and renames it on success. If encoding fails, the original file is preserved.

## Library Usage

The core functionality is available as a library crate (`slimg-core`):

```rust
use slimg_core::*;

// Decode an image file
let (image, format) = decode_file(Path::new("photo.jpg"))?;

// Convert to WebP with extend (add padding to make 1:1)
let result = convert(&image, &PipelineOptions {
    format: Format::WebP,
    quality: 80,
    resize: None,
    crop: None,
    extend: Some(ExtendMode::AspectRatio { width: 1, height: 1 }),
    fill_color: Some(FillColor::Solid([255, 255, 255, 255])),
})?;

// Save the result
result.save(Path::new("photo.webp"))?;
```
