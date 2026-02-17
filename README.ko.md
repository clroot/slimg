# slimg

빠른 이미지 최적화 CLI. 최신 코덱을 사용하여 이미지를 변환, 압축, 리사이즈합니다.

[English](./README.md)

## 지원 포맷

| 포맷 | 디코딩 | 인코딩 | 비고 |
|------|--------|--------|------|
| JPEG | O | O | MozJPEG 인코더로 뛰어난 압축률 |
| PNG | O | O | OxiPNG + Zopfli 압축 |
| WebP | O | O | libwebp 기반 손실 압축 |
| AVIF | macOS만 | O | ravif 인코더 (AV1 기반); 디코딩에 dav1d 필요 (macOS Homebrew) |
| QOI | O | O | 무손실, 빠른 인코딩/디코딩 |
| JPEG XL | O | X | 디코딩만 지원 (GPL 라이선스 제한) |

## 설치

### Homebrew (macOS / Linux)

```
brew install clroot/tap/slimg
```

### 빌드된 바이너리

[GitHub Releases](https://github.com/clroot/slimg/releases/latest)에서 다운로드:

| 플랫폼 | 파일 |
|--------|------|
| macOS (Apple Silicon) | `slimg-aarch64-apple-darwin.tar.xz` |
| macOS (Intel) | `slimg-x86_64-apple-darwin.tar.xz` |
| Linux (x86_64) | `slimg-x86_64-unknown-linux-gnu.tar.xz` |
| Linux (ARM64) | `slimg-aarch64-unknown-linux-gnu.tar.xz` |
| Windows (x86_64) | `slimg-x86_64-pc-windows-msvc.zip` |

### 소스에서 빌드

```
git clone https://github.com/clroot/slimg.git
cd slimg
cargo install --path cli
```

#### 빌드 요구사항

- Rust 1.85+ (edition 2024)
- C 컴파일러 (cc)
- nasm (MozJPEG / rav1e 어셈블리 최적화용)
- dav1d (macOS만, AVIF 디코딩용)

## 사용법

### convert

이미지를 다른 포맷으로 변환합니다.

```
# JPEG를 WebP로 변환 (기본 품질 80)
slimg convert photo.jpg --format webp

# AVIF로 변환 (품질 60)
slimg convert photo.png --format avif --quality 60

# 디렉토리 내 모든 이미지 변환
slimg convert ./images --format webp --output ./output --recursive

# 병렬 작업 수를 4개로 제한
slimg convert ./images --format webp --recursive --jobs 4
```

### optimize

같은 포맷으로 재인코딩하여 파일 크기를 줄입니다.

```
# JPEG 최적화 (품질 80)
slimg optimize photo.jpg

# 원본 파일 덮어쓰기
slimg optimize photo.jpg --overwrite

# 디렉토리 내 이미지 일괄 최적화
slimg optimize ./images --quality 70 --recursive

# 병렬 작업 수를 2개로 제한 (대용량 이미지에 유용)
slimg optimize ./images --recursive --jobs 2
```

### resize

이미지를 리사이즈합니다. 포맷 변환도 함께 가능합니다.

```
# 너비 기준 리사이즈 (비율 유지)
slimg resize photo.jpg --width 800

# 높이 기준 리사이즈
slimg resize photo.jpg --height 600

# 지정 영역 안에 맞추기 (비율 유지)
slimg resize photo.jpg --width 800 --height 600

# 배율로 리사이즈
slimg resize photo.jpg --scale 0.5

# 리사이즈 + 포맷 변환
slimg resize photo.jpg --width 400 --format webp --output thumb.webp
```

## 배치 처리

`--recursive` 옵션으로 디렉토리를 처리할 때, slimg은 [rayon](https://github.com/rayon-rs/rayon)을 통해 모든 CPU 코어를 활용합니다. `--jobs` 옵션으로 병렬 수를 제한할 수 있습니다 (대용량 이미지나 메모리가 제한된 환경에서 유용).

```
# 모든 코어 대신 4개 스레드만 사용
slimg convert ./images --format webp --recursive --jobs 4
```

**에러 처리** — 파일 처리 중 오류가 발생하면 해당 파일을 건너뛰고 나머지를 계속 처리합니다. 실패한 파일 목록은 마지막에 요약 출력됩니다.

**안전한 덮어쓰기** — `--overwrite` 사용 시, 임시 파일에 먼저 쓴 뒤 성공하면 이름을 변경합니다. 인코딩이 실패하면 원본 파일이 보존됩니다.

## 라이브러리

핵심 기능은 라이브러리 크레이트(`slimg-core`)로도 사용 가능합니다:

```rust
use slimg_core::*;

// 이미지 파일 디코딩
let (image, format) = decode_file(Path::new("photo.jpg"))?;

// WebP로 변환
let result = convert(&image, &PipelineOptions {
    format: Format::WebP,
    quality: 80,
    resize: None,
})?;

// 결과 저장
result.save(Path::new("photo.webp"))?;
```

## 라이선스

MIT OR Apache-2.0
