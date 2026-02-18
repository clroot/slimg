# Image Crop Feature Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** slimg에 좌표 기반(Region)과 비율 기반(AspectRatio) 이미지 크롭 기능을 추가한다.

**Architecture:** `resize.rs` 패턴을 따라 `crop.rs`에 `CropMode` enum + `crop()` 함수를 구현하고, `pipeline.rs`의 `PipelineOptions`에 crop 옵션을 추가한다. CLI에서는 `slimg crop` 서브커맨드를 독립적으로 제공하며, 배치 처리와 포맷 변환을 지원한다.

**Tech Stack:** Rust, clap (CLI), rayon (병렬 처리), UniFFI (Kotlin 바인딩)

---

### Task 1: Error variant 추가

**Files:**
- Modify: `crates/slimg-core/src/error.rs:6` (Resize 뒤에 Crop 추가)

**Step 1: Add Crop error variant**

`crates/slimg-core/src/error.rs`에 `Resize(String)` 바로 아래에 추가:

```rust
    #[error("crop error: {0}")]
    Crop(String),
```

**Step 2: Verify it compiles**

Run: `cargo check -p slimg-core`
Expected: success (새 variant는 아직 사용되지 않으므로 warning 가능)

**Step 3: Commit**

```bash
git add crates/slimg-core/src/error.rs
git commit -m "feat(core): add Crop error variant"
```

---

### Task 2: CropMode enum + calculate_crop_region 함수 (TDD)

**Files:**
- Create: `crates/slimg-core/src/crop.rs`
- Modify: `crates/slimg-core/src/lib.rs:5` (crop 모듈 등록)

**Step 1: Create crop.rs with CropMode, calculate_crop_region, and failing tests**

Create `crates/slimg-core/src/crop.rs`:

```rust
use crate::error::{Error, Result};

/// How to crop an image.
#[derive(Debug, Clone, PartialEq)]
pub enum CropMode {
    /// Extract a specific region: x, y offset with width x height.
    Region { x: u32, y: u32, width: u32, height: u32 },
    /// Crop to an aspect ratio (centered). Width and height define the ratio (e.g. 16:9).
    AspectRatio { width: u32, height: u32 },
}

/// Calculate the crop region (x, y, width, height) for a given image size and crop mode.
pub fn calculate_crop_region(
    img_w: u32,
    img_h: u32,
    mode: &CropMode,
) -> Result<(u32, u32, u32, u32)> {
    match *mode {
        CropMode::Region { x, y, width, height } => {
            if width == 0 || height == 0 {
                return Err(Error::Crop("crop dimensions must be non-zero".to_string()));
            }
            if x + width > img_w || y + height > img_h {
                return Err(Error::Crop(format!(
                    "crop region ({x},{y},{width},{height}) exceeds image bounds ({img_w}x{img_h})"
                )));
            }
            Ok((x, y, width, height))
        }
        CropMode::AspectRatio { width: rw, height: rh } => {
            if rw == 0 || rh == 0 {
                return Err(Error::Crop("aspect ratio must be non-zero".to_string()));
            }
            let target_ratio = rw as f64 / rh as f64;
            let img_ratio = img_w as f64 / img_h as f64;

            let (crop_w, crop_h) = if img_ratio > target_ratio {
                // Image is wider than target ratio — constrain by height
                let h = img_h;
                let w = (h as f64 * target_ratio).round() as u32;
                (w, h)
            } else {
                // Image is taller than target ratio — constrain by width
                let w = img_w;
                let h = (w as f64 / target_ratio).round() as u32;
                (w, h)
            };

            let x = (img_w - crop_w) / 2;
            let y = (img_h - crop_h) / 2;

            Ok((x, y, crop_w, crop_h))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_valid_crop() {
        let (x, y, w, h) = calculate_crop_region(200, 100, &CropMode::Region {
            x: 10, y: 20, width: 50, height: 30,
        }).unwrap();
        assert_eq!((x, y, w, h), (10, 20, 50, 30));
    }

    #[test]
    fn region_full_image() {
        let (x, y, w, h) = calculate_crop_region(200, 100, &CropMode::Region {
            x: 0, y: 0, width: 200, height: 100,
        }).unwrap();
        assert_eq!((x, y, w, h), (0, 0, 200, 100));
    }

    #[test]
    fn region_exceeds_bounds() {
        let result = calculate_crop_region(200, 100, &CropMode::Region {
            x: 150, y: 0, width: 100, height: 50,
        });
        assert!(result.is_err());
    }

    #[test]
    fn region_zero_width() {
        let result = calculate_crop_region(200, 100, &CropMode::Region {
            x: 0, y: 0, width: 0, height: 50,
        });
        assert!(result.is_err());
    }

    #[test]
    fn region_zero_height() {
        let result = calculate_crop_region(200, 100, &CropMode::Region {
            x: 0, y: 0, width: 50, height: 0,
        });
        assert!(result.is_err());
    }

    #[test]
    fn aspect_square_on_landscape() {
        // 200x100 image, 1:1 ratio -> 100x100 centered
        let (x, y, w, h) = calculate_crop_region(200, 100, &CropMode::AspectRatio {
            width: 1, height: 1,
        }).unwrap();
        assert_eq!((x, y, w, h), (50, 0, 100, 100));
    }

    #[test]
    fn aspect_square_on_portrait() {
        // 100x200 image, 1:1 ratio -> 100x100 centered
        let (x, y, w, h) = calculate_crop_region(100, 200, &CropMode::AspectRatio {
            width: 1, height: 1,
        }).unwrap();
        assert_eq!((x, y, w, h), (0, 50, 100, 100));
    }

    #[test]
    fn aspect_16_9_on_square() {
        // 100x100 image, 16:9 -> constrained by width: 100x56 centered
        let (x, y, w, h) = calculate_crop_region(100, 100, &CropMode::AspectRatio {
            width: 16, height: 9,
        }).unwrap();
        assert_eq!(w, 100);
        assert_eq!(h, 56); // 100 / (16/9) = 56.25 -> 56
        assert_eq!(x, 0);
        assert_eq!(y, 22); // (100 - 56) / 2 = 22
    }

    #[test]
    fn aspect_same_as_image() {
        // 200x100 image with 2:1 ratio -> full image
        let (x, y, w, h) = calculate_crop_region(200, 100, &CropMode::AspectRatio {
            width: 2, height: 1,
        }).unwrap();
        assert_eq!((x, y, w, h), (0, 0, 200, 100));
    }

    #[test]
    fn aspect_zero_ratio() {
        let result = calculate_crop_region(200, 100, &CropMode::AspectRatio {
            width: 0, height: 1,
        });
        assert!(result.is_err());
    }
}
```

**Step 2: Register crop module in lib.rs**

In `crates/slimg-core/src/lib.rs`, add `pub mod crop;` after `pub mod codec;` (line 1), and add `pub use crop::CropMode;` at the bottom with other re-exports.

```rust
pub mod codec;
pub mod crop;
pub mod error;
pub mod format;
pub mod pipeline;
pub mod resize;

pub use codec::{Codec, EncodeOptions, ImageData};
pub use crop::CropMode;
pub use error::{Error, Result};
pub use format::Format;
pub use pipeline::{
    PipelineOptions, PipelineResult, convert, decode, decode_file, optimize, output_path,
};
pub use resize::ResizeMode;
```

**Step 3: Run tests**

Run: `cargo test -p slimg-core crop`
Expected: all `calculate_crop_region` tests pass

**Step 4: Commit**

```bash
git add crates/slimg-core/src/crop.rs crates/slimg-core/src/lib.rs
git commit -m "feat(core): add CropMode enum and calculate_crop_region"
```

---

### Task 3: crop() 함수 구현 (TDD)

**Files:**
- Modify: `crates/slimg-core/src/crop.rs` (crop 함수 + 테스트 추가)

**Step 1: Add crop function and tests to crop.rs**

Add after `calculate_crop_region` function, before `#[cfg(test)]`:

```rust
use crate::codec::ImageData;

/// Crop an image according to the given mode.
pub fn crop(image: &ImageData, mode: &CropMode) -> Result<ImageData> {
    let (x, y, crop_w, crop_h) = calculate_crop_region(image.width, image.height, mode)?;

    let bytes_per_pixel = 4usize;
    let src_stride = image.width as usize * bytes_per_pixel;
    let dst_stride = crop_w as usize * bytes_per_pixel;

    let mut data = vec![0u8; crop_h as usize * dst_stride];

    for row in 0..crop_h as usize {
        let src_offset = (y as usize + row) * src_stride + x as usize * bytes_per_pixel;
        let dst_offset = row * dst_stride;
        data[dst_offset..dst_offset + dst_stride]
            .copy_from_slice(&image.data[src_offset..src_offset + dst_stride]);
    }

    Ok(ImageData::new(crop_w, crop_h, data))
}
```

Note: the `use crate::codec::ImageData;` import should be at the top of the file along with the other imports.

Add these tests inside `mod tests`:

```rust
    fn create_test_image(width: u32, height: u32) -> ImageData {
        let data = vec![128u8; (width * height * 4) as usize];
        ImageData::new(width, height, data)
    }

    #[test]
    fn crop_region_returns_correct_dimensions() {
        let img = create_test_image(200, 100);
        let result = crop(&img, &CropMode::Region {
            x: 10, y: 20, width: 50, height: 30,
        }).unwrap();
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 30);
        assert_eq!(result.data.len(), (50 * 30 * 4) as usize);
    }

    #[test]
    fn crop_aspect_returns_correct_dimensions() {
        let img = create_test_image(200, 100);
        let result = crop(&img, &CropMode::AspectRatio {
            width: 1, height: 1,
        }).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 100);
    }

    #[test]
    fn crop_preserves_pixel_data() {
        // Create 4x2 image with known pattern
        // Row 0: [R,G,B,A, R,G,B,A, R,G,B,A, R,G,B,A]
        // Row 1: [R,G,B,A, R,G,B,A, R,G,B,A, R,G,B,A]
        let mut data = vec![0u8; 4 * 2 * 4];
        for y in 0..2u32 {
            for x in 0..4u32 {
                let i = ((y * 4 + x) * 4) as usize;
                data[i] = x as u8;       // R = x coordinate
                data[i + 1] = y as u8;   // G = y coordinate
                data[i + 2] = 0;
                data[i + 3] = 255;
            }
        }
        let img = ImageData::new(4, 2, data);

        // Crop 2x1 region starting at (1,0)
        let result = crop(&img, &CropMode::Region {
            x: 1, y: 0, width: 2, height: 1,
        }).unwrap();

        assert_eq!(result.width, 2);
        assert_eq!(result.height, 1);
        // Should contain pixels at (1,0) and (2,0)
        assert_eq!(result.data[0], 1); // x=1
        assert_eq!(result.data[4], 2); // x=2
    }

    #[test]
    fn crop_full_image_returns_clone() {
        let img = create_test_image(100, 50);
        let result = crop(&img, &CropMode::Region {
            x: 0, y: 0, width: 100, height: 50,
        }).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
        assert_eq!(result.data, img.data);
    }
```

**Step 2: Run tests**

Run: `cargo test -p slimg-core crop`
Expected: all tests pass

**Step 3: Commit**

```bash
git add crates/slimg-core/src/crop.rs
git commit -m "feat(core): implement crop function with RGBA buffer slicing"
```

---

### Task 4: Pipeline 확장

**Files:**
- Modify: `crates/slimg-core/src/pipeline.rs:7-18` (import crop, add field, apply in convert)

**Step 1: Add crop to pipeline**

In `crates/slimg-core/src/pipeline.rs`:

1. Add import at top (line 7, after resize import):
```rust
use crate::crop::{self, CropMode};
```

2. Add `crop` field to `PipelineOptions` (after `resize` field, line 17):
```rust
    /// Optional crop to apply before encoding.
    pub crop: Option<CropMode>,
```

3. In `convert()` function, apply crop before resize (after line 58, before the resize match):
```rust
    let image = match &options.crop {
        Some(mode) => crop::crop(image, mode)?,
        None => image.clone(),
    };

    let image = match &options.resize {
        Some(mode) => resize::resize(&image, mode)?,
        None => image,
    };
```

Note: remove the existing resize match block and replace with the above. The first `image.clone()` is needed because crop takes `&ImageData`.

4. Fix all existing `PipelineOptions` usages to include `crop: None`:
   - In `pipeline.rs` tests (line 139)
   - In `crates/slimg-core/tests/integration.rs` (lines 23, 39, 57, 84)

**Step 2: Verify it compiles and all tests pass**

Run: `cargo test -p slimg-core`
Expected: all existing tests pass (no regressions)

**Step 3: Commit**

```bash
git add crates/slimg-core/src/pipeline.rs crates/slimg-core/tests/integration.rs
git commit -m "feat(core): add crop option to pipeline"
```

---

### Task 5: Pipeline crop 통합 테스트

**Files:**
- Modify: `crates/slimg-core/tests/integration.rs` (테스트 추가)

**Step 1: Add integration tests**

Append to `crates/slimg-core/tests/integration.rs`:

```rust
#[test]
fn convert_with_crop_region() {
    let image = create_test_image(); // 100x80

    let options = PipelineOptions {
        format: Format::Png,
        quality: 80,
        resize: None,
        crop: Some(CropMode::Region { x: 10, y: 10, width: 50, height: 40 }),
    };
    let result = convert(&image, &options).expect("PNG encode with crop failed");
    assert!(!result.data.is_empty());

    let (decoded, _) = decode(&result.data).expect("PNG decode failed");
    assert_eq!(decoded.width, 50);
    assert_eq!(decoded.height, 40);
}

#[test]
fn convert_with_crop_aspect_ratio() {
    let image = create_test_image(); // 100x80

    let options = PipelineOptions {
        format: Format::WebP,
        quality: 80,
        resize: None,
        crop: Some(CropMode::AspectRatio { width: 1, height: 1 }),
    };
    let result = convert(&image, &options).expect("WebP encode with crop failed");
    assert!(!result.data.is_empty());

    let (decoded, _) = decode(&result.data).expect("WebP decode failed");
    assert_eq!(decoded.width, 80);
    assert_eq!(decoded.height, 80);
}
```

**Step 2: Run integration tests**

Run: `cargo test -p slimg-core --test integration`
Expected: all tests pass

**Step 3: Commit**

```bash
git add crates/slimg-core/tests/integration.rs
git commit -m "test(core): add crop integration tests"
```

---

### Task 6: CLI crop 서브커맨드 구현

**Files:**
- Create: `cli/src/commands/crop.rs`
- Modify: `cli/src/commands/mod.rs:1-3` (모듈 등록)
- Modify: `cli/src/main.rs:16-41` (Commands enum + match)

**Step 1: Create crop command**

Create `cli/src/commands/crop.rs`:

```rust
use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use rayon::prelude::*;
use slimg_core::{CropMode, PipelineOptions, convert, decode_file, output_path};

use super::{
    ErrorCollector, FormatArg, collect_files, configure_thread_pool, make_progress_bar, safe_write,
};

#[derive(Debug, Args)]
pub struct CropArgs {
    /// Input file or directory
    pub input: PathBuf,

    /// Crop region: x,y,width,height (e.g. 100,50,800,600)
    #[arg(long, value_parser = parse_region, conflicts_with = "aspect")]
    pub region: Option<(u32, u32, u32, u32)>,

    /// Aspect ratio: width:height (e.g. 16:9, 1:1)
    #[arg(long, value_parser = parse_aspect, conflicts_with = "region")]
    pub aspect: Option<(u32, u32)>,

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

fn parse_region(s: &str) -> Result<(u32, u32, u32, u32), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return Err("expected format: x,y,width,height (e.g. 100,50,800,600)".to_string());
    }
    let nums: Vec<u32> = parts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            p.trim()
                .parse::<u32>()
                .map_err(|_| format!("invalid number at position {}: '{}'", i + 1, p))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((nums[0], nums[1], nums[2], nums[3]))
}

fn parse_aspect(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err("expected format: width:height (e.g. 16:9, 1:1)".to_string());
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
        return Err("aspect ratio values must be non-zero".to_string());
    }
    Ok((w, h))
}

fn build_crop_mode(args: &CropArgs) -> anyhow::Result<CropMode> {
    match (args.region, args.aspect) {
        (Some((x, y, w, h)), None) => Ok(CropMode::Region {
            x,
            y,
            width: w,
            height: h,
        }),
        (None, Some((w, h))) => Ok(CropMode::AspectRatio { width: w, height: h }),
        _ => anyhow::bail!("specify exactly one of --region or --aspect"),
    }
}

pub fn run(args: CropArgs) -> anyhow::Result<()> {
    let crop_mode = build_crop_mode(&args)?;
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

            let options = PipelineOptions {
                format: target_format,
                quality: args.quality,
                resize: None,
                crop: Some(crop_mode.clone()),
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
        anyhow::bail!("{fail_count} file(s) failed to crop");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_region_valid() {
        assert_eq!(parse_region("100,50,800,600"), Ok((100, 50, 800, 600)));
    }

    #[test]
    fn parse_region_with_spaces() {
        assert_eq!(parse_region("100, 50, 800, 600"), Ok((100, 50, 800, 600)));
    }

    #[test]
    fn parse_region_wrong_count() {
        assert!(parse_region("100,50,800").is_err());
    }

    #[test]
    fn parse_region_invalid_number() {
        assert!(parse_region("abc,50,800,600").is_err());
    }

    #[test]
    fn parse_aspect_valid() {
        assert_eq!(parse_aspect("16:9"), Ok((16, 9)));
    }

    #[test]
    fn parse_aspect_square() {
        assert_eq!(parse_aspect("1:1"), Ok((1, 1)));
    }

    #[test]
    fn parse_aspect_wrong_format() {
        assert!(parse_aspect("16-9").is_err());
    }

    #[test]
    fn parse_aspect_zero() {
        assert!(parse_aspect("0:9").is_err());
    }
}
```

**Step 2: Register crop module in mod.rs**

In `cli/src/commands/mod.rs`, add after line 3 (`pub mod resize;`):

```rust
pub mod crop;
```

**Step 3: Add Crop to Commands enum in main.rs**

In `cli/src/main.rs`, add to the `Commands` enum (after Resize):

```rust
    /// Crop image with optional format conversion
    Crop(commands::crop::CropArgs),
```

And add to the match block (after Resize):

```rust
        Commands::Crop(args) => commands::crop::run(args),
```

**Step 4: Verify it compiles and tests pass**

Run: `cargo test -p slimg-cli`
Expected: all tests pass (including new parse_region/parse_aspect tests)

**Step 5: Commit**

```bash
git add cli/src/commands/crop.rs cli/src/commands/mod.rs cli/src/main.rs
git commit -m "feat(cli): add crop subcommand with region and aspect ratio support"
```

---

### Task 7: FFI 바인딩 업데이트

**Files:**
- Modify: `crates/slimg-ffi/src/lib.rs`

**Step 1: Add CropMode and crop function to FFI**

In `crates/slimg-ffi/src/lib.rs`:

1. Add CropMode enum (after ResizeMode, around line 53):

```rust
/// How to crop an image.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum CropMode {
    /// Extract a specific region.
    Region { x: u32, y: u32, width: u32, height: u32 },
    /// Crop to an aspect ratio (centered).
    AspectRatio { width: u32, height: u32 },
}

impl CropMode {
    fn to_core(&self) -> slimg_core::CropMode {
        match self {
            CropMode::Region { x, y, width, height } => slimg_core::CropMode::Region {
                x: *x, y: *y, width: *width, height: *height,
            },
            CropMode::AspectRatio { width, height } => slimg_core::CropMode::AspectRatio {
                width: *width, height: *height,
            },
        }
    }
}
```

2. Add `crop` field to `PipelineOptions` (line 100, after resize):

```rust
    /// Optional crop to apply before encoding.
    pub crop: Option<CropMode>,
```

3. Update `convert` function to pass crop (around line 222):

```rust
    let core_options = slimg_core::PipelineOptions {
        format: options.format.to_core(),
        quality: options.quality,
        resize: options.resize.as_ref().map(|r| r.to_core()),
        crop: options.crop.as_ref().map(|c| c.to_core()),
    };
```

4. Add Crop error variant to `SlimgError` (after Resize):

```rust
    #[error("crop error: {message}")]
    Crop { message: String },
```

5. Add Crop error mapping in `From<slimg_core::Error>` (after Resize mapping):

```rust
            slimg_core::Error::Crop(s) => SlimgError::Crop { message: s },
```

6. Add standalone crop FFI function (after `convert` function):

```rust
/// Crop an image according to the given mode.
#[uniffi::export]
fn crop(image: &ImageData, mode: &CropMode) -> Result<ImageData, SlimgError> {
    let result = slimg_core::crop::crop(&image.to_core(), &mode.to_core())?;
    Ok(ImageData::from_core(result))
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p slimg-ffi`
Expected: compiles without errors

**Step 3: Commit**

```bash
git add crates/slimg-ffi/src/lib.rs
git commit -m "feat(ffi): expose CropMode and crop function via UniFFI"
```

---

### Task 8: Full test suite 검증

**Step 1: Run all tests**

Run: `cargo test --workspace`
Expected: all tests pass

**Step 2: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: no warnings

**Step 3: Verify CLI works end-to-end**

Run: `cargo run -p slimg-cli -- crop --help`
Expected: help output showing `--region`, `--aspect`, `--format`, `--quality`, `--output`, `--recursive`, `--jobs`, `--overwrite` options

**Step 4: Commit (if any fixes needed)**

Only if clippy fixes were needed.
