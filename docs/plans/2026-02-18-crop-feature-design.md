# Image Crop Feature Design

## Overview

slimg에 이미지 크롭 기능을 추가한다. 좌표 기반(Region)과 비율 기반(AspectRatio) 두 가지 모드를 지원하며, 독립 CLI 서브커맨드(`slimg crop`)로 제공한다.

## Approach

**접근 A: CropMode + 파이프라인 확장** 채택.

ResizeMode 패턴을 따라 CropMode enum을 core에 추가하고, 파이프라인에 crop 단계를 삽입한다. CLI, core 라이브러리, FFI 바인딩 모두에서 사용 가능.

## Core Layer

### CropMode (`crates/slimg-core/src/crop.rs`)

```rust
pub enum CropMode {
    /// 좌표 기반: x, y 오프셋에서 width x height 영역 추출
    Region { x: u32, y: u32, width: u32, height: u32 },

    /// 비율 기반: 이미지 중앙에서 주어진 비율로 최대 영역 추출
    AspectRatio { width: u32, height: u32 },
}
```

### crop 함수

```rust
pub fn crop(image: &ImageData, mode: &CropMode) -> Result<ImageData>
```

- RGBA 버퍼에서 행 단위 메모리 복사로 구현
- Region: 경계 초과 시 에러 반환
- AspectRatio: 중앙 고정, 이미지에서 해당 비율의 최대 영역 계산

### 파이프라인 확장 (`pipeline.rs`)

`PipelineOptions`에 `crop: Option<CropMode>` 필드 추가.

```
Decode → Crop (optional) → Encode
```

## CLI Layer

### 서브커맨드: `slimg crop`

```bash
# 좌표 기반
slimg crop photo.jpg --region 100,50,800,600
slimg crop photo.jpg --region 100,50,800,600 --output cropped.jpg

# 비율 기반 (중앙 고정)
slimg crop photo.jpg --aspect 16:9
slimg crop photo.jpg --aspect 1:1

# 포맷 변환 결합
slimg crop photo.jpg --aspect 1:1 --format webp --quality 80

# 배치 처리
slimg crop ./images --aspect 1:1 --recursive --format webp --jobs 4

# 출력 디렉토리
slimg crop ./images --aspect 1:1 --recursive --output ./thumbnails
```

### CropArgs

```rust
struct CropArgs {
    input: PathBuf,
    region: Option<(u32, u32, u32, u32)>,  // x,y,width,height
    aspect: Option<(u32, u32)>,             // W:H
    format: Option<Format>,
    quality: Option<u8>,
    output: Option<PathBuf>,
    recursive: bool,
    jobs: Option<usize>,
    overwrite: bool,
}
```

**검증:**
- `--region`과 `--aspect`는 상호 배타적, 둘 중 하나 필수
- `--region`: `x,y,w,h` 형식 (쉼표 구분)
- `--aspect`: `W:H` 형식 (콜론 구분)

### 공통 패턴 재사용

- `collect_files()`, `configure_thread_pool()`, `make_progress_bar()`, `safe_write()`, `ErrorCollector` 등 기존 유틸리티 사용
- progress bar + error summary 패턴 동일

## Test Strategy

### 단위 테스트 (`crop.rs`)

- Region: 정상 영역, 경계 초과 에러, width/height 0 에러
- AspectRatio: 가로/세로 이미지에 다양한 비율 적용, 결과 크기 검증
- 중앙 정렬 정확도 검증

### 통합 테스트 (`tests/integration.rs`)

- 크롭 후 포맷 변환
- 파이프라인 연동 (crop + encode)

### CLI 테스트

- `--region`, `--aspect` 파싱
- 상호 배타 옵션 에러

## FFI (`slimg-ffi`)

CropMode를 UniFFI enum으로 노출. Kotlin 바인딩에서 `crop()` 함수 사용 가능.

## Files Changed

| File | Change |
|------|--------|
| `crates/slimg-core/src/crop.rs` | New: CropMode + crop function |
| `crates/slimg-core/src/lib.rs` | Register crop module, re-export |
| `crates/slimg-core/src/pipeline.rs` | Add crop to PipelineOptions |
| `cli/src/commands/crop.rs` | New: crop subcommand |
| `cli/src/commands/mod.rs` | Register crop module |
| `cli/src/main.rs` | Add Crop to Commands enum |
| `crates/slimg-core/tests/integration.rs` | Crop integration tests |
| `crates/slimg-ffi/src/lib.rs` | Expose crop via FFI |
