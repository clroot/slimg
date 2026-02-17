# slimg — Image Optimization CLI & Library

## Overview

Squoosh(Google)가 유기된 이후, 그 핵심 기능(이미지 압축/최적화/포맷 변환)을 Rust CLI + 라이브러리로 재구현한다.

**목표:** Squoosh급 압축 품질을 CLI에서 사용할 수 있도록 한다.

## Project Structure

멀티 크레이트 워크스페이스.

```
slimg/
├── Cargo.toml              # workspace root
├── crates/
│   └── slimg-core/         # 라이브러리 크레이트 (코어 로직)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── codec/      # 코덱별 모듈
│           │   ├── mod.rs
│           │   ├── jpeg.rs    # MozJPEG
│           │   ├── png.rs     # OxiPNG
│           │   ├── webp.rs    # libwebp
│           │   ├── avif.rs    # ravif / libavif
│           │   ├── jxl.rs     # jxl-oxide
│           │   └── qoi.rs     # rapid-qoi
│           ├── resize.rs   # 리사이즈 엔진
│           └── pipeline.rs # 디코드 → 처리 → 인코드 파이프라인
└── cli/                    # CLI 바이너리
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## Supported Codecs

| Format | Encode | Decode | Crate | Notes |
|--------|--------|--------|-------|-------|
| JPEG | MozJPEG | mozjpeg | `mozjpeg-sys` | C binding, best quality |
| PNG | OxiPNG | image | `oxipng` | Pure Rust |
| WebP | libwebp | libwebp | `libwebp-sys` | C binding |
| AVIF | rav1e | libavif | `ravif` or `libavif` | C/Rust hybrid |
| JXL | jxl-oxide | jxl-oxide | `jxl-oxide` | Pure Rust (encoder limited) |
| QOI | rapid-qoi | rapid-qoi | `rapid-qoi` | Pure Rust |

### License Compatibility

모든 의존성은 BSD / MIT / Apache-2.0 조합. GPL 의존성(imagequant) 없음.
프로젝트 자체는 MIT 또는 Apache-2.0으로 배포 가능.

## CLI Interface

서브커맨드 기반 설계.

### convert — 포맷 변환 + 압축

```bash
slimg convert input.jpg -f webp -q 80
slimg convert input.png -f avif -q 75 -o output.avif
slimg convert ./images/ -f webp -q 80 -o ./output/
```

### optimize — 동일 포맷 최적화

```bash
slimg optimize input.jpg -q 80
slimg optimize input.png --level 4
slimg optimize ./photos/ --recursive
```

### resize — 리사이즈 (+ 선택적 포맷 변환)

```bash
slimg resize input.jpg --width 800 --height 600
slimg resize input.jpg --scale 0.5 -f webp
```

### Common Options

| Flag | Description | Default |
|------|-------------|---------|
| `-f, --format` | 출력 포맷 (jpeg, png, webp, avif, jxl, qoi) | 입력과 동일 |
| `-q, --quality` | 압축 품질 (0-100) | 80 |
| `-o, --output` | 출력 경로 (파일 또는 디렉토리) | 같은 디렉토리, 확장자 변경 |
| `--overwrite` | 원본 덮어쓰기 허용 | false |
| `--recursive` | 하위 디렉토리 포함 (배치) | false |

## Core Library API (slimg-core)

외부 Rust 프로젝트에서 import 가능한 API.

```rust
// 디코드
let image = slimg_core::decode(Path::new("input.jpg"))?;

// 변환
let result = slimg_core::convert(image, ConvertOptions {
    format: Format::WebP,
    quality: 80,
    resize: Some(Resize::Width(800)),
})?;

// 저장
result.save(Path::new("output.webp"))?;
```

## Pipeline

```
입력 (파일/디렉토리)
  → 포맷 감지 (매직 바이트 기반)
  → 디코드 → RGBA 버퍼
  → [리사이즈] (선택적)
  → 인코드 (목표 포맷 + 품질 옵션)
  → 출력 (파일)
```

## Output Behavior

- 기본: 같은 디렉토리에 확장자만 변경 (`photo.jpg` → `photo.webp`)
- `-o` 지정 시: 해당 경로에 저장
- 배치 모드: `-o`로 출력 디렉토리 지정, 원본 디렉토리 구조 유지
- 원본 덮어쓰기는 `--overwrite` 필요

## Build Requirements

C 바인딩 코덱 사용으로 빌드 시 필요:
- C/C++ 컴파일러 (gcc, clang)
- CMake (일부 코덱)
- nasm (MozJPEG SIMD)

## Non-Goals (v1)

- GUI / TUI 인터페이스
- imagequant (GPL3 라이선스)
- GIF 애니메이션 처리
- SVG 최적화
- 워터마크 / 메타데이터 편집
