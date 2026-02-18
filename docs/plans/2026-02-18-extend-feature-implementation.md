# Extend Feature Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 이미지에 여백을 추가해 비율이나 크기를 확장하는 `extend` 기능 구현 (크롭의 역연산)

**Architecture:** `extend.rs` 모듈을 `crop.rs`와 동일한 패턴으로 신규 생성. `ExtendMode` enum + `FillColor` enum + `extend()` 함수. 파이프라인에 Crop → Extend → Resize 순서로 통합. CLI는 `extend` 서브커맨드 추가. FFI도 동일하게 노출.

**Tech Stack:** Rust, Clap 4.5, rayon, slimg-core, UniFFI

---

### Task 1: Core - ExtendMode, FillColor 타입 + calculate_extend_region() 테스트

**Files:**
- Create: `crates/slimg-core/src/extend.rs`
- Modify: `crates/slimg-core/src/lib.rs:1-16`

**Step 1: extend.rs 생성 - 타입 정의와 calculate_extend_region 테스트 작성**

```rust
// crates/slimg-core/src/extend.rs

use crate::error::{Error, Result};

/// Fill color for the extended canvas area.
#[derive(Debug, Clone, PartialEq)]
pub enum FillColor {
    /// RGBA solid color.
    Solid([u8; 4]),
    /// Fully transparent (RGBA 0,0,0,0).
    Transparent,
}

impl FillColor {
    fn as_rgba(&self) -> [u8; 4] {
        match self {
            FillColor::Solid(c) => *c,
            FillColor::Transparent => [0, 0, 0, 0],
        }
    }
}

/// How to extend an image by adding padding around it.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtendMode {
    /// Extend to match an aspect ratio (centered). Width and height define the ratio (e.g. 1:1).
    AspectRatio { width: u32, height: u32 },
    /// Extend to an exact canvas size (centered). Must be >= image dimensions.
    Size { width: u32, height: u32 },
}

/// Calculate the canvas dimensions and image offset for a given image size and extend mode.
/// Returns (canvas_width, canvas_height, offset_x, offset_y).
pub fn calculate_extend_region(
    img_w: u32,
    img_h: u32,
    mode: &ExtendMode,
) -> Result<(u32, u32, u32, u32)> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── calculate_extend_region: AspectRatio ──

    #[test]
    fn aspect_square_on_landscape() {
        // 200x100 + 1:1 → canvas 200x200, offset (0, 50)
        let (cw, ch, ox, oy) =
            calculate_extend_region(200, 100, &ExtendMode::AspectRatio { width: 1, height: 1 })
                .unwrap();
        assert_eq!((cw, ch), (200, 200));
        assert_eq!((ox, oy), (0, 50));
    }

    #[test]
    fn aspect_square_on_portrait() {
        // 100x200 + 1:1 → canvas 200x200, offset (50, 0)
        let (cw, ch, ox, oy) =
            calculate_extend_region(100, 200, &ExtendMode::AspectRatio { width: 1, height: 1 })
                .unwrap();
        assert_eq!((cw, ch), (200, 200));
        assert_eq!((ox, oy), (50, 0));
    }

    #[test]
    fn aspect_16_9_on_square() {
        // 100x100 + 16:9 → canvas width 178 (100 * 16/9), height 100
        let (cw, ch, ox, oy) =
            calculate_extend_region(100, 100, &ExtendMode::AspectRatio { width: 16, height: 9 })
                .unwrap();
        assert_eq!(ch, 100);
        assert_eq!(cw, 178); // (100 * 16.0 / 9.0).round() = 178
        assert_eq!(oy, 0);
        assert_eq!(ox, 39); // (178 - 100) / 2
    }

    #[test]
    fn aspect_9_16_on_square() {
        // 100x100 + 9:16 → canvas width 100, height 178
        let (cw, ch, ox, oy) =
            calculate_extend_region(100, 100, &ExtendMode::AspectRatio { width: 9, height: 16 })
                .unwrap();
        assert_eq!(cw, 100);
        assert_eq!(ch, 178);
        assert_eq!(ox, 0);
        assert_eq!(oy, 39);
    }

    #[test]
    fn aspect_same_as_image() {
        // 200x100 + 2:1 → already matches, no-op
        let (cw, ch, ox, oy) =
            calculate_extend_region(200, 100, &ExtendMode::AspectRatio { width: 2, height: 1 })
                .unwrap();
        assert_eq!((cw, ch), (200, 100));
        assert_eq!((ox, oy), (0, 0));
    }

    #[test]
    fn aspect_zero_ratio_errors() {
        let result =
            calculate_extend_region(200, 100, &ExtendMode::AspectRatio { width: 0, height: 1 });
        assert!(result.is_err());
    }

    // ── calculate_extend_region: Size ──

    #[test]
    fn size_larger_canvas() {
        // 800x600 → 1000x1000
        let (cw, ch, ox, oy) =
            calculate_extend_region(800, 600, &ExtendMode::Size { width: 1000, height: 1000 })
                .unwrap();
        assert_eq!((cw, ch), (1000, 1000));
        assert_eq!((ox, oy), (100, 200));
    }

    #[test]
    fn size_same_as_image() {
        // 800x600 → 800x600, no-op
        let (cw, ch, ox, oy) =
            calculate_extend_region(800, 600, &ExtendMode::Size { width: 800, height: 600 })
                .unwrap();
        assert_eq!((cw, ch), (800, 600));
        assert_eq!((ox, oy), (0, 0));
    }

    #[test]
    fn size_only_width_larger() {
        // 800x600 → 1000x600
        let (cw, ch, ox, oy) =
            calculate_extend_region(800, 600, &ExtendMode::Size { width: 1000, height: 600 })
                .unwrap();
        assert_eq!((cw, ch), (1000, 600));
        assert_eq!((ox, oy), (100, 0));
    }

    #[test]
    fn size_smaller_than_image_errors() {
        let result =
            calculate_extend_region(800, 600, &ExtendMode::Size { width: 500, height: 500 });
        assert!(result.is_err());
    }

    #[test]
    fn size_width_smaller_errors() {
        let result =
            calculate_extend_region(800, 600, &ExtendMode::Size { width: 700, height: 600 });
        assert!(result.is_err());
    }

    #[test]
    fn size_zero_errors() {
        let result =
            calculate_extend_region(800, 600, &ExtendMode::Size { width: 0, height: 0 });
        assert!(result.is_err());
    }
}
```

**Step 2: lib.rs에 모듈 등록**

`crates/slimg-core/src/lib.rs`에 추가:

```rust
pub mod extend;
// pub use 라인에 추가:
pub use extend::{ExtendMode, FillColor};
```

**Step 3: 테스트 실행 - 실패 확인**

Run: `cargo test -p slimg-core extend`
Expected: FAIL — `todo!()` panic

**Step 4: calculate_extend_region 구현**

```rust
pub fn calculate_extend_region(
    img_w: u32,
    img_h: u32,
    mode: &ExtendMode,
) -> Result<(u32, u32, u32, u32)> {
    match *mode {
        ExtendMode::AspectRatio {
            width: rw,
            height: rh,
        } => {
            if rw == 0 || rh == 0 {
                return Err(Error::Extend("aspect ratio must be non-zero".to_string()));
            }

            let target_ratio = rw as f64 / rh as f64;
            let img_ratio = img_w as f64 / img_h as f64;

            let (canvas_w, canvas_h) = if img_ratio < target_ratio {
                // Image is taller than target → extend width
                let w = (img_h as f64 * target_ratio).round() as u32;
                (w, img_h)
            } else {
                // Image is wider than target → extend height
                let h = (img_w as f64 / target_ratio).round() as u32;
                (img_w, h)
            };

            let off_x = (canvas_w - img_w) / 2;
            let off_y = (canvas_h - img_h) / 2;

            Ok((canvas_w, canvas_h, off_x, off_y))
        }
        ExtendMode::Size { width, height } => {
            if width == 0 || height == 0 {
                return Err(Error::Extend(
                    "target size must be non-zero".to_string(),
                ));
            }
            if width < img_w || height < img_h {
                return Err(Error::Extend(format!(
                    "target size ({width}x{height}) is smaller than image ({img_w}x{img_h})"
                )));
            }

            let off_x = (width - img_w) / 2;
            let off_y = (height - img_h) / 2;

            Ok((width, height, off_x, off_y))
        }
    }
}
```

**Step 5: 테스트 실행 - 통과 확인**

Run: `cargo test -p slimg-core extend`
Expected: ALL PASS

**Step 6: Commit**

```bash
git add crates/slimg-core/src/extend.rs crates/slimg-core/src/lib.rs
git commit -m "feat(core): add ExtendMode, FillColor types and calculate_extend_region"
```

---

### Task 2: Core - extend() 함수 + 픽셀 테스트

**Files:**
- Modify: `crates/slimg-core/src/extend.rs`

**Step 1: extend() 함수 테스트 추가**

`extend.rs`의 `#[cfg(test)] mod tests` 블록에 추가:

```rust
    use crate::codec::ImageData;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let data = vec![128u8; (width * height * 4) as usize];
        ImageData::new(width, height, data)
    }

    // ── extend() ──

    #[test]
    fn extend_returns_correct_dimensions() {
        let img = create_test_image(200, 100);
        let result = extend(
            &img,
            &ExtendMode::AspectRatio { width: 1, height: 1 },
            &FillColor::Solid([255, 255, 255, 255]),
        )
        .unwrap();
        assert_eq!(result.width, 200);
        assert_eq!(result.height, 200);
        assert_eq!(result.data.len(), (200 * 200 * 4) as usize);
    }

    #[test]
    fn extend_fills_with_solid_color() {
        let img = create_test_image(2, 2);
        let result = extend(
            &img,
            &ExtendMode::Size { width: 4, height: 4 },
            &FillColor::Solid([255, 0, 0, 255]),
        )
        .unwrap();
        // Top-left corner (0,0) should be fill color (red)
        assert_eq!(result.data[0], 255); // R
        assert_eq!(result.data[1], 0);   // G
        assert_eq!(result.data[2], 0);   // B
        assert_eq!(result.data[3], 255); // A
    }

    #[test]
    fn extend_fills_with_transparent() {
        let img = create_test_image(2, 2);
        let result = extend(
            &img,
            &ExtendMode::Size { width: 4, height: 4 },
            &FillColor::Transparent,
        )
        .unwrap();
        // Top-left corner should be transparent
        assert_eq!(&result.data[0..4], &[0, 0, 0, 0]);
    }

    #[test]
    fn extend_preserves_pixel_data() {
        // Create 2x1 image: pixel(0,0)=[10,20,30,255] pixel(1,0)=[40,50,60,255]
        let data = vec![10, 20, 30, 255, 40, 50, 60, 255];
        let img = ImageData::new(2, 1, data);

        // Extend to 4x3 → original centered at offset (1, 1)
        let result = extend(
            &img,
            &ExtendMode::Size { width: 4, height: 3 },
            &FillColor::Solid([0, 0, 0, 0]),
        )
        .unwrap();

        // Row 1 (y=1), pixel at x=1 should be [10,20,30,255]
        let stride = 4 * 4; // 4 pixels * 4 bytes
        let offset = 1 * stride + 1 * 4; // row 1, col 1
        assert_eq!(&result.data[offset..offset + 4], &[10, 20, 30, 255]);

        // Row 1, pixel at x=2 should be [40,50,60,255]
        let offset2 = 1 * stride + 2 * 4;
        assert_eq!(&result.data[offset2..offset2 + 4], &[40, 50, 60, 255]);
    }

    #[test]
    fn extend_noop_when_already_matching() {
        let img = create_test_image(200, 100);
        let result = extend(
            &img,
            &ExtendMode::AspectRatio { width: 2, height: 1 },
            &FillColor::Solid([255, 255, 255, 255]),
        )
        .unwrap();
        assert_eq!(result.width, 200);
        assert_eq!(result.height, 100);
        assert_eq!(result.data, img.data);
    }
```

**Step 2: 테스트 실행 - 실패 확인**

Run: `cargo test -p slimg-core extend`
Expected: FAIL — `extend` function not found

**Step 3: extend() 함수 구현**

```rust
use crate::codec::ImageData;

/// Extend an image by adding padding around it.
pub fn extend(image: &ImageData, mode: &ExtendMode, fill: &FillColor) -> Result<ImageData> {
    let (canvas_w, canvas_h, off_x, off_y) =
        calculate_extend_region(image.width, image.height, mode)?;

    // No-op: canvas matches image
    if canvas_w == image.width && canvas_h == image.height {
        return Ok(image.clone());
    }

    let bytes_per_pixel = 4usize;
    let canvas_stride = canvas_w as usize * bytes_per_pixel;
    let src_stride = image.width as usize * bytes_per_pixel;

    // Fill canvas with background color
    let fill_rgba = fill.as_rgba();
    let mut data = vec![0u8; canvas_h as usize * canvas_stride];
    for pixel in data.chunks_exact_mut(bytes_per_pixel) {
        pixel.copy_from_slice(&fill_rgba);
    }

    // Copy original image rows into canvas at offset
    for row in 0..image.height as usize {
        let src_offset = row * src_stride;
        let dst_offset = (off_y as usize + row) * canvas_stride + off_x as usize * bytes_per_pixel;
        data[dst_offset..dst_offset + src_stride]
            .copy_from_slice(&image.data[src_offset..src_offset + src_stride]);
    }

    Ok(ImageData::new(canvas_w, canvas_h, data))
}
```

**Step 4: 테스트 실행 - 통과 확인**

Run: `cargo test -p slimg-core extend`
Expected: ALL PASS

**Step 5: Commit**

```bash
git add crates/slimg-core/src/extend.rs
git commit -m "feat(core): implement extend() function with pixel copy"
```

---

### Task 3: Core - Error variant + Pipeline 통합

**Files:**
- Modify: `crates/slimg-core/src/error.rs:1-33`
- Modify: `crates/slimg-core/src/pipeline.rs:1-153`

**Step 1: Error::Extend 변형 추가**

`crates/slimg-core/src/error.rs`에 `Crop` 뒤에 추가:

```rust
    #[error("extend error: {0}")]
    Extend(String),
```

**Step 2: pipeline.rs에 extend 통합**

`crates/slimg-core/src/pipeline.rs` 변경:

import에 추가:
```rust
use crate::extend::{self, ExtendMode, FillColor};
```

`PipelineOptions`에 필드 추가:
```rust
pub struct PipelineOptions {
    pub format: Format,
    pub quality: u8,
    pub resize: Option<ResizeMode>,
    pub crop: Option<CropMode>,
    pub extend: Option<ExtendMode>,
    pub fill_color: Option<FillColor>,
}
```

`convert()` 함수에서 crop 후, resize 전에 extend 단계 추가:
```rust
    let image = match &options.crop {
        Some(mode) => crop::crop(image, mode)?,
        None => image.clone(),
    };

    let image = match &options.extend {
        Some(mode) => {
            let fill = options.fill_color.clone().unwrap_or(FillColor::Solid([255, 255, 255, 255]));
            extend::extend(&image, mode, &fill)?
        }
        None => image,
    };

    let image = match &options.resize {
        Some(mode) => resize::resize(&image, mode)?,
        None => image,
    };
```

**Step 3: 기존 테스트에 새 필드 반영**

pipeline.rs의 기존 테스트 `jxl_encode_returns_error`에서 PipelineOptions에 필드 추가:
```rust
    let options = PipelineOptions {
        format: Format::Jxl,
        quality: 80,
        resize: None,
        crop: None,
        extend: None,
        fill_color: None,
    };
```

CLI의 crop.rs, convert.rs, resize.rs에서 PipelineOptions 생성 부분에도 동일하게 `extend: None, fill_color: None` 추가.

**Step 4: 컴파일 확인 + 전체 테스트**

Run: `cargo test`
Expected: ALL PASS

**Step 5: Commit**

```bash
git add crates/slimg-core/src/error.rs crates/slimg-core/src/pipeline.rs crates/slimg-core/src/extend.rs cli/src/commands/crop.rs cli/src/commands/convert.rs cli/src/commands/resize.rs cli/src/commands/optimize.rs
git commit -m "feat(core): add Error::Extend and integrate extend into pipeline"
```

---

### Task 4: CLI - extend 서브커맨드

**Files:**
- Create: `cli/src/commands/extend.rs`
- Modify: `cli/src/commands/mod.rs:1-5`
- Modify: `cli/src/main.rs:1-45`

**Step 1: commands/mod.rs에 parse_size 추가 + 모듈 등록**

```rust
pub mod extend;
```

`parse_size` 함수 추가 (mod.rs 하단, tests 위):
```rust
/// Parse "WxH" size string (e.g. "1920x1080").
pub(crate) fn parse_size(s: &str) -> std::result::Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err("expected format: WIDTHxHEIGHT (e.g. 1920x1080)".to_string());
    }
    let w: u32 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("invalid width: '{}'", parts[0]))?;
    let h: u32 = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("invalid height: '{}'", parts[1]))?;
    if w == 0 || h == 0 {
        return Err("size values must be non-zero".to_string());
    }
    Ok((w, h))
}
```

**Step 2: extend.rs CLI 커맨드 생성**

```rust
// cli/src/commands/extend.rs

use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use rayon::prelude::*;
use slimg_core::{ExtendMode, FillColor, PipelineOptions, convert, decode_file, output_path};

use super::{
    ErrorCollector, FormatArg, collect_files, configure_thread_pool, make_progress_bar,
    parse_size, safe_write,
};

#[derive(Debug, Args)]
pub struct ExtendArgs {
    /// Input file or directory
    pub input: PathBuf,

    /// Aspect ratio: width:height (e.g. 1:1, 16:9)
    #[arg(long, value_parser = super::crop::parse_aspect, conflicts_with = "size")]
    pub aspect: Option<(u32, u32)>,

    /// Target size: WIDTHxHEIGHT (e.g. 1920x1080)
    #[arg(long, value_parser = parse_size, conflicts_with = "aspect")]
    pub size: Option<(u32, u32)>,

    /// Fill color as hex (e.g. '#FFFFFF', '000000'). Default: white.
    #[arg(long, conflicts_with = "transparent")]
    pub color: Option<String>,

    /// Use transparent background (for formats with alpha support)
    #[arg(long, conflicts_with = "color")]
    pub transparent: bool,

    /// Output format (defaults to input format)
    #[arg(short, long)]
    pub format: Option<FormatArg>,

    /// Encoding quality (0-100)
    #[arg(short, long, default_value_t = 80)]
    pub quality: u8,

    /// Output path (file or directory)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Process subdirectories recursively
    #[arg(long)]
    pub recursive: bool,

    /// Number of parallel jobs (defaults to CPU count)
    #[arg(short, long)]
    pub jobs: Option<usize>,

    /// Overwrite existing files
    #[arg(long)]
    pub overwrite: bool,
}

fn parse_hex_color(s: &str) -> anyhow::Result<[u8; 4]> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 && s.len() != 8 {
        anyhow::bail!("expected 6 or 8 hex digits (e.g. 'FF0000' or 'FF0000FF')");
    }
    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;
    let a = if s.len() == 8 {
        u8::from_str_radix(&s[6..8], 16)?
    } else {
        255
    };
    Ok([r, g, b, a])
}

fn build_extend_mode(args: &ExtendArgs) -> anyhow::Result<ExtendMode> {
    match (args.aspect, args.size) {
        (Some((w, h)), None) => Ok(ExtendMode::AspectRatio { width: w, height: h }),
        (None, Some((w, h))) => Ok(ExtendMode::Size { width: w, height: h }),
        _ => anyhow::bail!("specify exactly one of --aspect or --size"),
    }
}

fn build_fill_color(args: &ExtendArgs, format: slimg_core::Format) -> anyhow::Result<FillColor> {
    if args.transparent {
        if format == slimg_core::Format::Jpeg {
            eprintln!(
                "warning: JPEG does not support transparency, using white background"
            );
            return Ok(FillColor::Solid([255, 255, 255, 255]));
        }
        return Ok(FillColor::Transparent);
    }

    match &args.color {
        Some(hex) => Ok(FillColor::Solid(parse_hex_color(hex)?)),
        None => Ok(FillColor::Solid([255, 255, 255, 255])),
    }
}

pub fn run(args: ExtendArgs) -> anyhow::Result<()> {
    let extend_mode = build_extend_mode(&args)?;
    let files = collect_files(&args.input, args.recursive)?;

    if files.is_empty() {
        anyhow::bail!("no image files found in {}", args.input.display());
    }

    configure_thread_pool(args.jobs)?;

    let pb = make_progress_bar(files.len());
    let errors = ErrorCollector::new();

    files.par_iter().for_each(|file| {
        let result: anyhow::Result<()> = (|| {
            let original_size = std::fs::metadata(file)?.len();
            let (image, src_format) =
                decode_file(file).with_context(|| format!("{}", file.display()))?;

            let target_format = args.format.map(|f| f.into_format()).unwrap_or(src_format);

            if !target_format.can_encode() {
                anyhow::bail!("cannot encode to {} format", target_format.extension());
            }

            let fill = build_fill_color(&args, target_format)?;

            let options = PipelineOptions {
                format: target_format,
                quality: args.quality,
                resize: None,
                crop: None,
                extend: Some(extend_mode.clone()),
                fill_color: Some(fill),
            };

            let result =
                convert(&image, &options).with_context(|| format!("{}", file.display()))?;

            let out = output_path(file, target_format, args.output.as_deref());
            safe_write(&out, &result.data, args.overwrite)?;

            let new_size = result.data.len() as u64;
            let ratio = if original_size > 0 {
                (new_size as f64 / original_size as f64) * 100.0
            } else {
                0.0
            };

            pb.println(format!(
                "{} -> {} ({} -> {} bytes, {:.1}%)",
                file.display(),
                out.display(),
                original_size,
                new_size,
                ratio,
            ));

            Ok(())
        })();

        if let Err(e) = result {
            errors.push(file, &e);
        }
        pb.inc(1);
    });

    let fail_count = errors.summarize(&pb);
    pb.finish_and_clear();

    if fail_count > 0 {
        anyhow::bail!("{fail_count} file(s) failed to extend");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color_6_digits() {
        assert_eq!(parse_hex_color("FF0000").unwrap(), [255, 0, 0, 255]);
    }

    #[test]
    fn parse_hex_color_with_hash() {
        assert_eq!(parse_hex_color("#00FF00").unwrap(), [0, 255, 0, 255]);
    }

    #[test]
    fn parse_hex_color_8_digits() {
        assert_eq!(parse_hex_color("FF000080").unwrap(), [255, 0, 0, 128]);
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert!(parse_hex_color("xyz").is_err());
    }

    #[test]
    fn parse_size_valid() {
        assert_eq!(super::super::parse_size("1920x1080"), Ok((1920, 1080)));
    }

    #[test]
    fn parse_size_zero() {
        assert!(super::super::parse_size("0x100").is_err());
    }

    #[test]
    fn parse_size_wrong_format() {
        assert!(super::super::parse_size("1920-1080").is_err());
    }
}
```

**Step 3: main.rs에 Extend 서브커맨드 등록**

`cli/src/main.rs`의 Commands enum에 추가:
```rust
    /// Extend image by adding padding with optional format conversion
    Extend(commands::extend::ExtendArgs),
```

match 분기에 추가:
```rust
    Commands::Extend(args) => commands::extend::run(args),
```

**Step 4: crop.rs의 parse_aspect를 pub(crate)로 변경**

`cli/src/commands/crop.rs`에서 `fn parse_aspect`를 `pub(crate) fn parse_aspect`로 변경하여 extend.rs에서 재사용.

**Step 5: 컴파일 + 테스트**

Run: `cargo test`
Expected: ALL PASS

Run: `cargo build -p slimg-cli`
Expected: BUILD SUCCESS

**Step 6: Commit**

```bash
git add cli/src/commands/extend.rs cli/src/commands/mod.rs cli/src/commands/crop.rs cli/src/main.rs
git commit -m "feat(cli): add extend subcommand with aspect ratio and size modes"
```

---

### Task 5: FFI - ExtendMode, FillColor 노출

**Files:**
- Modify: `crates/slimg-ffi/src/lib.rs`

**Step 1: FFI 타입 추가**

`CropMode` impl 뒤에 추가:

```rust
/// How to extend an image by adding padding.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum ExtendMode {
    /// Extend to match an aspect ratio (centered).
    AspectRatio { width: u32, height: u32 },
    /// Extend to an exact canvas size (centered).
    Size { width: u32, height: u32 },
}

impl ExtendMode {
    fn to_core(&self) -> slimg_core::ExtendMode {
        match self {
            ExtendMode::AspectRatio { width, height } => slimg_core::ExtendMode::AspectRatio {
                width: *width, height: *height,
            },
            ExtendMode::Size { width, height } => slimg_core::ExtendMode::Size {
                width: *width, height: *height,
            },
        }
    }
}

/// Fill color for extended canvas area.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum FillColor {
    /// RGBA solid color.
    Solid { r: u8, g: u8, b: u8, a: u8 },
    /// Fully transparent.
    Transparent,
}

impl FillColor {
    fn to_core(&self) -> slimg_core::FillColor {
        match self {
            FillColor::Solid { r, g, b, a } => slimg_core::FillColor::Solid([*r, *g, *b, *a]),
            FillColor::Transparent => slimg_core::FillColor::Transparent,
        }
    }
}
```

**Step 2: PipelineOptions에 extend/fill_color 필드 추가**

```rust
pub struct PipelineOptions {
    pub format: Format,
    pub quality: u8,
    pub resize: Option<ResizeMode>,
    pub crop: Option<CropMode>,
    pub extend: Option<ExtendMode>,
    pub fill_color: Option<FillColor>,
}
```

`convert` 함수의 `core_options` 생성부에 반영:

```rust
    let core_options = slimg_core::PipelineOptions {
        format: options.format.to_core(),
        quality: options.quality,
        resize: options.resize.as_ref().map(|r| r.to_core()),
        crop: options.crop.as_ref().map(|c| c.to_core()),
        extend: options.extend.as_ref().map(|e| e.to_core()),
        fill_color: options.fill_color.as_ref().map(|f| f.to_core()),
    };
```

**Step 3: extend FFI 함수 추가**

`crop` 함수 뒤에 추가:

```rust
/// Extend an image by adding padding around it.
#[uniffi::export]
fn extend(image: &ImageData, mode: &ExtendMode, fill: &FillColor) -> Result<ImageData, SlimgError> {
    let result = slimg_core::extend::extend(&image.to_core(), &mode.to_core(), &fill.to_core())?;
    Ok(ImageData::from_core(result))
}
```

**Step 4: SlimgError에 Extend 변형 추가**

```rust
    #[error("extend error: {message}")]
    Extend { message: String },
```

`From<slimg_core::Error>` impl에 추가:
```rust
    slimg_core::Error::Extend(s) => SlimgError::Extend { message: s },
```

**Step 5: 컴파일 확인**

Run: `cargo build -p slimg-ffi`
Expected: BUILD SUCCESS

**Step 6: Commit**

```bash
git add crates/slimg-ffi/src/lib.rs
git commit -m "feat(ffi): expose ExtendMode, FillColor and extend function via UniFFI"
```

---

### Task 6: 문서 업데이트 + 최종 검증

**Files:**
- Modify: `docs/usage.md` (extend 사용법 섹션 추가)

**Step 1: 전체 테스트 실행**

Run: `cargo test`
Expected: ALL PASS

**Step 2: CLI 실제 동작 확인**

Run: `cargo run -p slimg-cli -- extend --help`
Expected: extend 서브커맨드 도움말 출력

**Step 3: usage.md에 extend 섹션 추가**

기존 Crop 섹션과 동일한 형식으로 Extend 섹션 추가.

**Step 4: Commit**

```bash
git add docs/usage.md
git commit -m "docs: add extend feature usage guide"
```
