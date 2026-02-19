# JPEG XL Encoding Support

## Summary

libjxl (BSD-3-Clause) C library에 대한 직접 바인딩을 통해 slimg에 JPEG XL 인코딩을 추가한다.
기존 디코딩(image crate)은 유지하고, 인코딩만 libjxl로 구현한다.

## Motivation

- slimg은 JXL 디코딩만 지원하고 인코딩은 `EncodingNotSupported` 반환
- 기존 Rust JXL 인코더 crate들이 미성숙(v0.1~0.3)하거나 GPL-3.0(jpegxl-rs)
- libjxl 레퍼런스 구현(BSD-3-Clause)에 직접 바인딩하면 라이선스 호환 + 안정성 확보

## Clean-room Design

GPL-3.0인 `jpegxl-sys`/`jpegxl-rs`의 코드를 참조하지 않는다.
libjxl 공식 C 헤더(BSD-3-Clause)만 기반으로 독자 구현한다.

| 항목 | jpegxl-sys (GPL) | libjxl-enc-sys (ours, MIT) |
|------|-----------------|---------------------------|
| 범위 | 디코더+인코더+전체 API | 인코더 전용 (allowlist 필터링) |
| 빌드 | 자체 스크립트 | cmake crate 기반 독자 구현 |
| sys crate 구조 | 다중 모듈 | flat (bindgen output) |
| safe wrapper 위치 | jpegxl-rs (별도 crate) | slimg-core 내부 모듈 |
| 디코딩 | 포함 | 미포함 (기존 image crate 유지) |

## Architecture

### Crate Structure

```
crates/libjxl-enc-sys/           # 새 crate: raw FFI bindings (MIT)
  Cargo.toml                      # build-dep: cmake, bindgen
  build.rs                        # libjxl CMake 빌드 + bindgen
  src/lib.rs                      # include!(bindgen output)
  libjxl/                         # git submodule (BSD-3-Clause)

crates/slimg-core/src/codec/
  jxl/                            # 기존 jxl.rs -> 디렉토리로 변환
    mod.rs                        # JxlCodec (Codec trait 구현)
    encoder.rs                    # safe encoder wrapper
    types.rs                      # quality -> distance 매핑, 설정 변환
```

### Build System (build.rs)

1. cmake crate로 libjxl 서브모듈을 static library로 빌드
2. bindgen으로 인코더 API 바인딩 생성
3. `cargo:rustc-link-lib=static=jxl` 링킹

CMake 최소 빌드 옵션:

```
BUILD_TESTING=OFF
JPEGXL_ENABLE_TOOLS=OFF
JPEGXL_ENABLE_DOXYGEN=OFF
JPEGXL_ENABLE_MANPAGES=OFF
JPEGXL_ENABLE_BENCHMARK=OFF
JPEGXL_ENABLE_EXAMPLES=OFF
JPEGXL_ENABLE_SJPEG=OFF
JPEGXL_ENABLE_JPEGLI=OFF
BUILD_SHARED_LIBS=OFF
```

### Bindgen Allowlist

```rust
bindgen::Builder::default()
    .header("libjxl/lib/include/jxl/encode.h")
    .header("libjxl/lib/include/jxl/types.h")
    .header("libjxl/lib/include/jxl/codestream_header.h")
    .header("libjxl/lib/include/jxl/color_encoding.h")
    .allowlist_function("JxlEncoder.*")
    .allowlist_function("JxlColorEncodingSetToSRGB")
    .allowlist_function("JxlEncoderDistanceFromQuality")
    .allowlist_type("JxlBasicInfo|JxlPixelFormat|JxlDataType|JxlEndianness")
    .allowlist_type("JxlColorEncoding|JxlEncoder.*|JxlBool")
    .allowlist_var("JXL_TRUE|JXL_FALSE|JXL_ENC_.*")
```

### Safe Wrapper (slimg-core/src/codec/jxl/)

**encoder.rs** - libjxl encoder의 safe Rust 래퍼:

```rust
pub struct JxlEncoder { /* raw pointer + Drop */ }

impl JxlEncoder {
    pub fn new() -> Result<Self>;
    pub fn encode_rgba(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        config: &JxlEncodeConfig,
    ) -> Result<Vec<u8>>;
}
```

**types.rs** - quality 매핑:

```rust
pub struct JxlEncodeConfig {
    pub lossless: bool,
    pub distance: f32,
}

impl JxlEncodeConfig {
    pub fn from_quality(quality: u8) -> Self {
        if quality == 100 {
            return Self { lossless: true, distance: 0.0 };
        }
        let distance = unsafe {
            libjxl_enc_sys::JxlEncoderDistanceFromQuality(quality as f32)
        };
        Self { lossless: false, distance }
    }
}
```

**mod.rs** - Codec trait 구현:

```rust
impl Codec for JxlCodec {
    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        // 기존 image crate 디코딩 유지 (변경 없음)
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        let config = JxlEncodeConfig::from_quality(options.quality);
        let encoder = JxlEncoder::new()?;
        encoder.encode_rgba(&image.data, image.width, image.height, &config)
    }
}
```

### Other Changes

- `format.rs`: `can_encode()` - JXL을 `true`로 변경
- `dist-workspace.toml`: system dependencies에 cmake 추가
- Workspace `Cargo.toml`: `libjxl-enc-sys` 멤버 추가

## Encoding Features

- Lossy 인코딩: quality 0~99 (distance via `JxlEncoderDistanceFromQuality`)
- Lossless 인코딩: quality 100 (distance=0.0 + lossless flag)
- 기본 포함 (feature flag 없음)
- Vendored 빌드 (libjxl git submodule)

## CI Impact

현재 NASM, meson, ninja는 이미 설치됨. CMake + C++ 컴파일러 추가 필요.
GitHub Actions runner에 기본 탑재되어 있으므로 `dist-workspace.toml`에 cmake만 추가.

## License

- libjxl: BSD-3-Clause (MIT 호환)
- libjxl-enc-sys: MIT (slimg과 동일)
