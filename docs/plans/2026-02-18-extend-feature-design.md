# Extend Feature Design

## Summary

이미지에 여백을 추가해 비율이나 크기를 확장하는 기능. 크롭(잘라내기)의 반대 개념으로, 원본 이미지를 중앙에 배치하고 주변에 단색 또는 투명 여백을 채운다.

## Motivation

- 이미지를 1:1 등 특정 비율로 만들 때, 크롭 대신 여백 추가로 원본을 보존하고 싶은 경우
- SNS 업로드, 썸네일 생성 등에서 일관된 비율/크기가 필요하지만 원본 손실은 원치 않는 경우

## Design

### Core Types (`crates/slimg-core/src/extend.rs`)

```rust
pub enum FillColor {
    Solid([u8; 4]),   // RGBA
    Transparent,       // [0,0,0,0]
}

pub enum ExtendMode {
    AspectRatio { width: u32, height: u32 },
    Size { width: u32, height: u32 },
}
```

- **AspectRatio**: 목표 비율에 맞게 부족한 쪽에 여백 추가. 원본 중앙 배치.
  - 예: 800x600 + 1:1 → 800x800 (상하 100px 여백)
- **Size**: 목표 크기로 확장. 원본보다 작은 크기는 에러.
  - 예: 800x600 + 1000x1000 → 1000x1000 (좌우 100px, 상하 200px 여백)

### Core Functions

```rust
fn calculate_extend_region(img_w: u32, img_h: u32, mode: &ExtendMode)
    -> Result<(u32, u32, u32, u32)>
    // Returns: (canvas_width, canvas_height, offset_x, offset_y)

pub fn extend(image: &RawImage, mode: &ExtendMode, fill: &FillColor)
    -> Result<RawImage>
```

`extend()` 동작:
1. `calculate_extend_region()`으로 캔버스 크기와 오프셋 계산
2. `canvas_w * canvas_h * 4` 바이트 버퍼를 fill_color로 초기화
3. 원본 픽셀을 (off_x, off_y) 위치에 row-by-row 복사

### Pipeline Integration (`pipeline.rs`)

실행 순서: **Decode → Crop → Extend → Resize → Encode**

```rust
pub struct PipelineOptions {
    pub format: Format,
    pub quality: u8,
    pub resize: Option<ResizeMode>,
    pub crop: Option<CropMode>,
    pub extend: Option<ExtendMode>,      // NEW
    pub fill_color: Option<FillColor>,   // NEW
}
```

### CLI (`cli/src/commands/extend.rs`)

```bash
slimg extend photo.jpg --aspect 1:1
slimg extend photo.jpg --size 1920x1080
slimg extend photo.jpg --aspect 1:1 --color '#000000'
slimg extend photo.jpg --aspect 1:1 --transparent
slimg extend photo.jpg --aspect 1:1 --transparent --format png
slimg extend ./images --aspect 1:1 --recursive --output ./squared --jobs 4
```

```rust
pub struct ExtendArgs {
    pub input: PathBuf,
    pub aspect: Option<(u32, u32)>,     // conflicts_with size
    pub size: Option<(u32, u32)>,       // conflicts_with aspect
    pub color: Option<String>,          // hex color, default #FFFFFF
    pub transparent: bool,              // conflicts_with color
    pub format: Option<FormatArg>,
    pub quality: u8,                    // default 80
    pub output: Option<PathBuf>,
    pub recursive: bool,
    pub jobs: Option<usize>,
    pub overwrite: bool,
}
```

상호 배타:
- `--aspect` vs `--size` (둘 중 하나 필수)
- `--color` vs `--transparent` (둘 다 없으면 기본 흰색)

### Error Handling

`Error::Extend(String)` 변형 추가.

| 상황 | 처리 |
|------|------|
| Size 모드에서 원본보다 작은 크기 | 에러 반환 |
| AspectRatio에서 이미 해당 비율 | 원본 그대로 반환 (no-op) |
| JPEG + --transparent | 경고 출력, 흰색으로 폴백 |

### FFI (`crates/slimg-ffi/src/lib.rs`)

```rust
pub enum FfiExtendMode {
    AspectRatio { width: u32, height: u32 },
    Size { width: u32, height: u32 },
}

pub enum FfiFillColor {
    Solid { r: u8, g: u8, b: u8, a: u8 },
    Transparent,
}
```

### File Changes

| Layer | File | Change |
|-------|------|--------|
| Core | `extend.rs` (new) | `ExtendMode`, `FillColor`, `extend()` |
| Core | `pipeline.rs` | Add extend/fill_color to `PipelineOptions` |
| Core | `error.rs` | Add `Error::Extend` |
| Core | `lib.rs` | Add `pub mod extend` |
| CLI | `commands/extend.rs` (new) | `ExtendArgs`, `run()` |
| CLI | `commands/mod.rs` | Add `parse_size()` |
| CLI | `main.rs` | Add `Extend` subcommand |
| FFI | `lib.rs` | Expose `FfiExtendMode`, `FfiFillColor` |

### Test Strategy

- **Core unit tests**: canvas size calculation, offset calculation, pixel copy verification, boundary conditions
- **CLI tests**: `parse_size()` validation, mutual exclusivity
- **Integration tests**: actual image file extend → decode and verify dimensions
