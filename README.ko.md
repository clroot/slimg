# slimg

[![CI](https://github.com/clroot/slimg/actions/workflows/release.yml/badge.svg)](https://github.com/clroot/slimg/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/slimg)](https://crates.io/crates/slimg)
[![PyPI](https://img.shields.io/pypi/v/slimg)](https://pypi.org/project/slimg/)
[![Maven Central](https://img.shields.io/maven-central/v/io.clroot.slimg/slimg-kotlin)](https://central.sonatype.com/artifact/io.clroot.slimg/slimg-kotlin)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.85+-orange.svg)](https://www.rust-lang.org)

빠른 이미지 최적화 CLI. 최신 코덱을 사용하여 이미지를 변환, 압축, 리사이즈, 크롭, 확장합니다.

[English](./README.md)

## 지원 포맷

| 포맷 | 디코딩 | 인코딩 | 비고 |
|------|--------|--------|------|
| JPEG | O | O | MozJPEG 인코더로 뛰어난 압축률 |
| PNG | O | O | OxiPNG + Zopfli 압축 |
| WebP | O | O | libwebp 기반 손실 압축 |
| AVIF | O | O | ravif 인코더; dav1d 디코더 (정적 링크) |
| QOI | O | O | 무손실, 빠른 인코딩/디코딩 |
| JPEG XL | O | O | libjxl 인코더/디코더 |

## 설치

### Cargo (crates.io)

```
cargo install slimg
```

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
- meson + ninja (dav1d AVIF 디코더 소스 빌드용)
- `SYSTEM_DEPS_DAV1D_BUILD_INTERNAL=always` 설정으로 dav1d를 소스에서 빌드

## 사용법

상세 사용 가이드는 [docs/usage.ko.md](./docs/usage.ko.md)를 참고하세요.

```bash
# 포맷 변환
slimg convert photo.jpg --format webp

# 최적화 (같은 포맷으로 재인코딩)
slimg optimize photo.jpg --quality 70

# 리사이즈
slimg resize photo.jpg --width 800

# 좌표로 크롭
slimg crop photo.jpg --region 100,50,800,600

# 비율로 크롭 (중앙 기준)
slimg crop photo.jpg --aspect 16:9

# 여백 추가로 정사각형 만들기
slimg extend photo.jpg --aspect 1:1

# 투명 배경으로 확장
slimg extend photo.png --aspect 1:1 --transparent

# 배치 처리 + 포맷 변환
slimg convert ./images --format webp --output ./output --recursive --jobs 4
```

## 벤치마크

모든 코덱과 파이프라인 연산의 상세 성능 측정 결과는 [docs/benchmarks.md](./docs/benchmarks.md)를 참고하세요.

## 언어 바인딩

| 언어 | 패키지 | 플랫폼 |
|------|--------|--------|
| [Kotlin/JVM](./bindings/kotlin/) | `io.clroot.slimg:slimg-kotlin` | macOS, Linux, Windows |
| [Python](./bindings/python/) | `slimg` | macOS, Linux, Windows |

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
    crop: None,
    extend: None,
    fill_color: None,
})?;

// 결과 저장
result.save(Path::new("photo.webp"))?;
```

## 라이선스

MIT
