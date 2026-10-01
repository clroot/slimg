---
name: slimg
description: Convert, compress, resize, crop, and pad images with the slimg CLI. Supports JPEG, PNG, WebP, AVIF, JXL, and QOI, plus batch processing of directories. Use when the user wants to convert, compress, optimize, resize, crop, or pad images; asks to make images smaller or web-ready; or needs batch processing of an image directory.
---

# slimg

A fast image optimization CLI built on proven encoders: MozJPEG (JPEG), OxiPNG + Zopfli (PNG), libwebp, ravif (AVIF), and libjxl. Reach for it before ImageMagick or sharp scripts.

```bash
which slimg || brew install clroot/tap/slimg   # or: cargo install slimg
```

## Commands

| Command | What it does | Required options |
| --- | --- | --- |
| `convert` | Convert to another format | `--format` |
| `optimize` | Re-encode in the same format to reduce size | `--overwrite` or `--output` |
| `resize` | Resize (+ optional format conversion) | one of `--width` / `--height` / `--scale` |
| `crop` | Crop (+ optional format conversion) | `--region` or `--aspect` |
| `extend` | Pad to an aspect ratio or size (+ optional format conversion) | `--aspect` or `--size` |

Run `slimg <command> --help` for full options. `slimg completions <shell>` generates shell completions.

## Common options

| Option | Description |
| --- | --- |
| `-f, --format` | `jpeg` `png` `webp` `avif` `jxl` `qoi` |
| `-q, --quality` | Encoding quality 0–100 (default 80) |
| `-o, --output` | Output file or directory |
| `--recursive` | Include subdirectories |
| `-j, --jobs` | Number of parallel jobs (default: all cores) |
| `--overwrite` | Allow overwriting existing files |

## Output path rules — the most common mistake

- No `--output` → the result is written **next to the input** with only the extension changed (`photo.jpg` → `photo.webp`).
- For operations that keep the format (e.g. resize or crop without `--format`), omitting `--output` means output path = input path. Without `--overwrite` this errors; with it, **the original is replaced**. Pass `--output` to keep the original.
- When processing multiple files, `--output` must be a directory (created if missing). For a single file it can be a file path.

## Per-command constraints

- **optimize** — Replaces files in place by default, so either `--overwrite` or `--output` is required. In place, a file is skipped if the result is not smaller.
- **resize** — Passing both `--width` and `--height` fits the image within those bounds while preserving aspect ratio. It does not force exact dimensions.
- **crop** — `--region x,y,w,h` or `--aspect w:h` (centered). Mutually exclusive; one is required.
- **extend** — `--aspect w:h` or `--size WxH`. Mutually exclusive; one is required. `--size` must be greater than or equal to the original. `--color '#RRGGBB'` (default white) and `--transparent` are also mutually exclusive. `--transparent` with JPEG output falls back to white with a warning.

## Safety and batch behavior

- No existing file is ever overwritten without `--overwrite`. Writes go to a temp file and are then renamed, so an interrupted run never leaves a corrupt file.
- In a batch, failed files are skipped and processing continues; a failure summary is printed at the end and the exit code is non-zero. In scripts, check the exit code to determine success.
- Symlinked directories are not followed during recursion (symlinked files are processed).
- If memory becomes a concern when batching large images, lower parallelism, e.g. `--jobs 2`.

## Choosing a format

| Goal | Recommendation |
| --- | --- |
| Web delivery, compatibility first | `webp` (q 75–85) |
| Web delivery, smallest size (accepts ~3x slower encoding) | `avif` (q 50–65) |
| Lossless archival | `png` (slow but smallest) / `qoi` (100x faster encoding) |
| Next-gen high quality / lossless | `jxl` |

The default quality of 80 works well in most cases. To shrink photos further without visible degradation, try going down to around webp 75 or avif 60.

## Recipes

```bash
# Batch convert a whole directory to WebP for the web
slimg convert ./images --format webp --output ./dist --recursive

# Shrink files in their original format (in place)
slimg optimize ./images --quality 70 --recursive --overwrite

# Thumbnail: 400px wide WebP
slimg resize photo.jpg --width 400 --format webp --output thumb.webp

# Center-crop to 16:9 and convert to AVIF
slimg crop photo.jpg --aspect 16:9 --format avif

# Pad onto a square transparent canvas (logos, avatars)
slimg extend logo.png --aspect 1:1 --transparent --output logo-square.png
```
