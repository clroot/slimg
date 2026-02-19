# JPEG XL Encoding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** libjxl (BSD-3-Clause)에 대한 clean-room 바인딩으로 slimg에 JXL 인코딩 지원 추가

**Architecture:** workspace에 `libjxl-enc-sys` crate를 추가하여 libjxl C 인코더를 vendored static 빌드 + bindgen 바인딩. `slimg-core`의 `codec/jxl/` 모듈에서 safe wrapper를 통해 Codec trait 구현.

**Tech Stack:** libjxl (C++/CMake), cmake crate, bindgen, git submodule

**Clean-room 원칙:** GPL-3.0인 `jpegxl-sys`/`jpegxl-rs` 코드를 절대 참조하지 않는다. libjxl 공식 C 헤더(BSD-3-Clause)만 기반으로 독자 구현.

---

### Task 1: libjxl git submodule 추가

**Files:**
- Create: `crates/libjxl-enc-sys/` (directory)
- Create: `.gitmodules` entry

**Step 1: 디렉토리 생성 및 submodule 추가**

```bash
mkdir -p crates/libjxl-enc-sys
git submodule add https://github.com/libjxl/libjxl.git crates/libjxl-enc-sys/libjxl
```

**Step 2: libjxl의 third-party 서브모듈 초기화**

libjxl은 highway, brotli 등을 third_party/에서 참조한다.

```bash
cd crates/libjxl-enc-sys/libjxl
git submodule update --init --recursive --depth 1 third_party/highway third_party/brotli third_party/skcms
cd ../../..
```

**Step 3: 안정 태그로 체크아웃**

```bash
cd crates/libjxl-enc-sys/libjxl
git checkout v0.11.1
cd ../../..
```

**Step 4: 커밋**

```bash
git add .gitmodules crates/libjxl-enc-sys/libjxl
git commit -m "chore: add libjxl v0.11.1 as git submodule"
```

---

### Task 2: libjxl-enc-sys crate 스캐폴딩

**Files:**
- Create: `crates/libjxl-enc-sys/Cargo.toml`
- Create: `crates/libjxl-enc-sys/src/lib.rs`
- Modify: `Cargo.toml` (workspace members)

**Step 1: Cargo.toml 작성**

```toml
# crates/libjxl-enc-sys/Cargo.toml
[package]
name = "libjxl-enc-sys"
version = "0.1.0"
edition = "2024"
license = "MIT"
description = "Minimal FFI bindings to libjxl encoder (vendored, BSD-3-Clause)"
publish = false
links = "jxl_enc"

[build-dependencies]
cmake = "0.1"
bindgen = "0.71"
```

**Step 2: 빈 lib.rs 작성**

```rust
// crates/libjxl-enc-sys/src/lib.rs
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

**Step 3: workspace에 멤버 추가**

`Cargo.toml`의 `members`에 `"crates/libjxl-enc-sys"` 추가.

**Step 4: 커밋**

```bash
git add crates/libjxl-enc-sys/Cargo.toml crates/libjxl-enc-sys/src/lib.rs Cargo.toml
git commit -m "chore: scaffold libjxl-enc-sys crate"
```

---

### Task 3: build.rs 구현 (CMake + bindgen)

**Files:**
- Create: `crates/libjxl-enc-sys/build.rs`

**Step 1: build.rs 작성**

```rust
// crates/libjxl-enc-sys/build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let dst = cmake::Config::new("libjxl")
        .define("BUILD_TESTING", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("JPEGXL_ENABLE_TOOLS", "OFF")
        .define("JPEGXL_ENABLE_DOXYGEN", "OFF")
        .define("JPEGXL_ENABLE_MANPAGES", "OFF")
        .define("JPEGXL_ENABLE_BENCHMARK", "OFF")
        .define("JPEGXL_ENABLE_EXAMPLES", "OFF")
        .define("JPEGXL_ENABLE_SJPEG", "OFF")
        .define("JPEGXL_ENABLE_JPEGLI", "OFF")
        .define("JPEGXL_ENABLE_OPENEXR", "OFF")
        .define("JPEGXL_ENABLE_TCMALLOC", "OFF")
        .define("JPEGXL_BUNDLE_LIBPNG", "OFF")
        .define("JPEGXL_ENABLE_SKCMS", "ON")
        .build();

    let lib_dir = dst.join("lib");
    let lib64_dir = dst.join("lib64");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-search=native={}", lib64_dir.display());

    // libjxl core + encoder
    println!("cargo:rustc-link-lib=static=jxl");
    println!("cargo:rustc-link-lib=static=jxl_enc");
    println!("cargo:rustc-link-lib=static=jxl_cms");

    // libjxl vendored dependencies
    println!("cargo:rustc-link-lib=static=hwy");
    println!("cargo:rustc-link-lib=static=brotlienc");
    println!("cargo:rustc-link-lib=static=brotlidec");
    println!("cargo:rustc-link-lib=static=brotlicommon");
    println!("cargo:rustc-link-lib=static=skcms");

    // C++ standard library
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target_os.as_str() {
        "macos" | "ios" => println!("cargo:rustc-link-lib=c++"),
        "windows" => {} // MSVC links C++ runtime automatically
        _ => println!("cargo:rustc-link-lib=stdc++"),
    }

    // bindgen
    let include_dir = dst.join("include");
    let src_include = PathBuf::from("libjxl/lib/include");

    let bindings = bindgen::Builder::default()
        .header(src_include.join("jxl/encode.h").to_str().unwrap())
        .header(src_include.join("jxl/types.h").to_str().unwrap())
        .header(src_include.join("jxl/codestream_header.h").to_str().unwrap())
        .header(src_include.join("jxl/color_encoding.h").to_str().unwrap())
        .clang_arg(format!("-I{}", src_include.display()))
        .clang_arg(format!("-I{}", include_dir.display()))
        // Encoder functions only
        .allowlist_function("JxlEncoderCreate")
        .allowlist_function("JxlEncoderDestroy")
        .allowlist_function("JxlEncoderReset")
        .allowlist_function("JxlEncoderSetBasicInfo")
        .allowlist_function("JxlEncoderSetColorEncoding")
        .allowlist_function("JxlEncoderFrameSettingsCreate")
        .allowlist_function("JxlEncoderSetFrameDistance")
        .allowlist_function("JxlEncoderSetFrameLossless")
        .allowlist_function("JxlEncoderFrameSettingsSetOption")
        .allowlist_function("JxlEncoderAddImageFrame")
        .allowlist_function("JxlEncoderCloseInput")
        .allowlist_function("JxlEncoderProcessOutput")
        .allowlist_function("JxlEncoderDistanceFromQuality")
        .allowlist_function("JxlColorEncodingSetToSRGB")
        // Types
        .allowlist_type("JxlBasicInfo")
        .allowlist_type("JxlPixelFormat")
        .allowlist_type("JxlDataType")
        .allowlist_type("JxlEndianness")
        .allowlist_type("JxlColorEncoding")
        .allowlist_type("JxlEncoderStatus")
        .allowlist_type("JxlEncoderFrameSettingId")
        .allowlist_type("JxlEncoder")
        .allowlist_type("JxlEncoderFrameSettings")
        .generate()
        .expect("failed to generate libjxl bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings");
}
```

**Step 2: 빌드 확인**

```bash
cargo build -p libjxl-enc-sys
```

Expected: 첫 빌드는 libjxl CMake 빌드로 수 분 소요. 성공 시 static 라이브러리 + bindings.rs 생성.

> **Note:** 링킹 라이브러리 이름(`jxl`, `jxl_enc`, `jxl_cms`, `hwy` 등)은 libjxl 빌드 결과에 따라 조정이 필요할 수 있다. 빌드 실패 시 `target/debug/build/libjxl-enc-sys-*/out/lib/` 디렉토리에서 실제 생성된 `.a` 파일명을 확인하여 맞춘다.

**Step 3: 커밋**

```bash
git add crates/libjxl-enc-sys/build.rs
git commit -m "feat(libjxl-enc-sys): implement build.rs with CMake + bindgen"
```

---

### Task 4: jxl.rs를 jxl/ 디렉토리 모듈로 변환

**Files:**
- Delete: `crates/slimg-core/src/codec/jxl.rs`
- Create: `crates/slimg-core/src/codec/jxl/mod.rs`
- Create: `crates/slimg-core/src/codec/jxl/types.rs`
- Create: `crates/slimg-core/src/codec/jxl/encoder.rs`

**Step 1: 디렉토리 생성 및 기존 코드 이동**

```bash
mkdir -p crates/slimg-core/src/codec/jxl
mv crates/slimg-core/src/codec/jxl.rs crates/slimg-core/src/codec/jxl/mod.rs
```

**Step 2: 빈 모듈 파일 생성**

```rust
// crates/slimg-core/src/codec/jxl/types.rs
use crate::error::Result;

/// JXL 인코딩 설정.
pub(crate) struct EncodeConfig {
    pub lossless: bool,
    pub distance: f32,
}

impl EncodeConfig {
    pub fn from_quality(quality: u8) -> Self {
        if quality >= 100 {
            return Self {
                lossless: true,
                distance: 0.0,
            };
        }
        let distance =
            unsafe { libjxl_enc_sys::JxlEncoderDistanceFromQuality(quality as f32) };
        Self {
            lossless: false,
            distance,
        }
    }
}
```

```rust
// crates/slimg-core/src/codec/jxl/encoder.rs
// Safe wrapper는 Task 6에서 구현. 컴파일 확인용 빈 파일.
```

**Step 3: mod.rs에 모듈 선언 추가**

`mod.rs` 상단에 추가:

```rust
mod encoder;
mod types;
```

**Step 4: slimg-core에 libjxl-enc-sys 의존성 추가**

`crates/slimg-core/Cargo.toml`의 `[dependencies]`에 추가:

```toml
libjxl-enc-sys = { path = "../libjxl-enc-sys" }
```

**Step 5: 빌드 확인**

```bash
cargo build -p slimg-core
```

**Step 6: 커밋**

```bash
git add crates/slimg-core/src/codec/jxl/ crates/slimg-core/Cargo.toml
git rm crates/slimg-core/src/codec/jxl.rs 2>/dev/null || true
git commit -m "refactor(slimg-core): convert jxl.rs to jxl/ module directory"
```

---

### Task 5: safe encoder wrapper 구현

**Files:**
- Modify: `crates/slimg-core/src/codec/jxl/encoder.rs`

**Step 1: encoder.rs 작성**

```rust
// crates/slimg-core/src/codec/jxl/encoder.rs
use std::ptr;

use libjxl_enc_sys::*;

use crate::error::{Error, Result};

use super::types::EncodeConfig;

/// Safe wrapper around libjxl encoder.
pub(crate) struct Encoder {
    ptr: *mut JxlEncoder,
}

impl Encoder {
    /// Create a new JXL encoder instance.
    pub fn new() -> Result<Self> {
        let ptr = unsafe { JxlEncoderCreate(ptr::null()) };
        if ptr.is_null() {
            return Err(Error::Encode("failed to create JXL encoder".into()));
        }
        Ok(Self { ptr })
    }

    /// Encode RGBA pixel data into JXL format.
    pub fn encode_rgba(
        &mut self,
        pixels: &[u8],
        width: u32,
        height: u32,
        config: &EncodeConfig,
    ) -> Result<Vec<u8>> {
        unsafe {
            JxlEncoderReset(self.ptr);

            self.set_basic_info(width, height, config)?;
            self.set_color_encoding()?;

            let frame_settings = JxlEncoderFrameSettingsCreate(self.ptr, ptr::null());
            if frame_settings.is_null() {
                return Err(Error::Encode(
                    "failed to create frame settings".into(),
                ));
            }

            self.configure_frame(frame_settings, config)?;
            self.add_frame(frame_settings, pixels, width, height)?;

            JxlEncoderCloseInput(self.ptr);

            self.process_output()
        }
    }

    unsafe fn set_basic_info(
        &self,
        width: u32,
        height: u32,
        config: &EncodeConfig,
    ) -> Result<()> {
        let mut info: JxlBasicInfo = std::mem::zeroed();
        info.xsize = width;
        info.ysize = height;
        info.bits_per_sample = 8;
        info.exponent_bits_per_sample = 0;
        info.num_color_channels = 3;
        info.num_extra_channels = 1;
        info.alpha_bits = 8;
        info.alpha_exponent_bits = 0;
        info.uses_original_profile = if config.lossless { 1 } else { 0 };

        check_status(
            JxlEncoderSetBasicInfo(self.ptr, &info),
            "set basic info",
        )
    }

    unsafe fn set_color_encoding(&self) -> Result<()> {
        let mut color: JxlColorEncoding = std::mem::zeroed();
        JxlColorEncodingSetToSRGB(&mut color, 0); // is_gray = false
        check_status(
            JxlEncoderSetColorEncoding(self.ptr, &color),
            "set color encoding",
        )
    }

    unsafe fn configure_frame(
        &self,
        settings: *mut JxlEncoderFrameSettings,
        config: &EncodeConfig,
    ) -> Result<()> {
        if config.lossless {
            check_status(
                JxlEncoderSetFrameLossless(settings, 1),
                "set lossless",
            )?;
        }
        check_status(
            JxlEncoderSetFrameDistance(settings, config.distance),
            "set distance",
        )
    }

    unsafe fn add_frame(
        &self,
        settings: *mut JxlEncoderFrameSettings,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Result<()> {
        let format = JxlPixelFormat {
            num_channels: 4,
            data_type: JXL_TYPE_UINT8,
            endianness: JXL_NATIVE_ENDIAN,
            align: 0,
        };

        let expected = (width as usize) * (height as usize) * 4;
        debug_assert_eq!(pixels.len(), expected);

        check_status(
            JxlEncoderAddImageFrame(
                settings,
                &format,
                pixels.as_ptr().cast(),
                pixels.len(),
            ),
            "add image frame",
        )
    }

    unsafe fn process_output(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; 64 * 1024]; // 64 KB initial
        let mut all_output = Vec::new();

        loop {
            let mut next_out = buffer.as_mut_ptr();
            let mut avail_out = buffer.len();

            let status =
                JxlEncoderProcessOutput(self.ptr, &mut next_out, &mut avail_out);

            let written = buffer.len() - avail_out;
            all_output.extend_from_slice(&buffer[..written]);

            match status {
                JXL_ENC_SUCCESS => return Ok(all_output),
                JXL_ENC_NEED_MORE_OUTPUT => continue,
                _ => {
                    return Err(Error::Encode("JXL encoding failed".into()))
                }
            }
        }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe {
            JxlEncoderDestroy(self.ptr);
        }
    }
}

/// Check a `JxlEncoderStatus` and convert to `Result`.
unsafe fn check_status(status: JxlEncoderStatus, context: &str) -> Result<()> {
    if status == JXL_ENC_SUCCESS {
        Ok(())
    } else {
        Err(Error::Encode(format!("jxl {context}: status {status}")))
    }
}
```

> **Note:** bindgen이 생성하는 상수 이름(`JXL_ENC_SUCCESS`, `JXL_TYPE_UINT8`, `JXL_NATIVE_ENDIAN` 등)은 실제 생성 결과에 따라 다를 수 있다. Task 3 빌드 성공 후 `target/debug/build/libjxl-enc-sys-*/out/bindings.rs`에서 실제 이름을 확인하여 맞춘다.

**Step 2: 빌드 확인**

```bash
cargo build -p slimg-core
```

**Step 3: 커밋**

```bash
git add crates/slimg-core/src/codec/jxl/encoder.rs
git commit -m "feat(slimg-core): implement safe JXL encoder wrapper"
```

---

### Task 6: Codec trait 인코딩 구현 + format.rs 업데이트

**Files:**
- Modify: `crates/slimg-core/src/codec/jxl/mod.rs`
- Modify: `crates/slimg-core/src/format.rs`

**Step 1: mod.rs에서 인코딩 구현**

```rust
// crates/slimg-core/src/codec/jxl/mod.rs
mod encoder;
mod types;

use crate::error::{Error, Result};
use crate::format::Format;

use super::{Codec, EncodeOptions, ImageData};

/// JXL codec backed by libjxl.
pub struct JxlCodec;

impl Codec for JxlCodec {
    fn format(&self) -> Format {
        Format::Jxl
    }

    fn decode(&self, data: &[u8]) -> Result<ImageData> {
        let img = image::load_from_memory(data)
            .map_err(|e| Error::Decode(format!("jxl decode: {e}")))?;

        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();

        Ok(ImageData::new(width, height, rgba.into_raw()))
    }

    fn encode(&self, image: &ImageData, options: &EncodeOptions) -> Result<Vec<u8>> {
        let config = types::EncodeConfig::from_quality(options.quality);
        let mut enc = encoder::Encoder::new()?;
        enc.encode_rgba(&image.data, image.width, image.height, &config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let size = (width * height * 4) as usize;
        let mut data = vec![0u8; size];
        for y in 0..height {
            for x in 0..width {
                let i = ((y * width + x) * 4) as usize;
                data[i] = (x * 255 / width) as u8;
                data[i + 1] = (y * 255 / height) as u8;
                data[i + 2] = 128;
                data[i + 3] = 255;
            }
        }
        ImageData::new(width, height, data)
    }

    #[test]
    fn encode_produces_valid_jxl() {
        let codec = JxlCodec;
        let image = create_test_image(64, 48);
        let options = EncodeOptions { quality: 80 };

        let encoded = codec.encode(&image, &options).expect("encode failed");
        assert!(encoded.len() > 2, "encoded data too short");

        // JXL bare codestream: [0xFF, 0x0A] or container: [0x00,0x00,0x00,0x0C]
        let is_jxl = (encoded[0] == 0xFF && encoded[1] == 0x0A)
            || (encoded.len() >= 8
                && encoded[..4] == [0x00, 0x00, 0x00, 0x0C]
                && &encoded[4..8] == b"JXL ");
        assert!(is_jxl, "output is not valid JXL");
    }

    #[test]
    fn encode_and_decode_roundtrip() {
        let codec = JxlCodec;
        let original = create_test_image(64, 48);
        let options = EncodeOptions { quality: 90 };

        let encoded = codec.encode(&original, &options).expect("encode failed");
        let decoded = codec.decode(&encoded).expect("decode failed");

        assert_eq!(decoded.width, original.width);
        assert_eq!(decoded.height, original.height);
        assert_eq!(
            decoded.data.len(),
            (decoded.width * decoded.height * 4) as usize
        );
    }

    #[test]
    fn lower_quality_produces_smaller_file() {
        let codec = JxlCodec;
        let image = create_test_image(128, 96);

        let high = codec
            .encode(&image, &EncodeOptions { quality: 95 })
            .expect("encode q95 failed");
        let low = codec
            .encode(&image, &EncodeOptions { quality: 20 })
            .expect("encode q20 failed");

        assert!(
            low.len() < high.len(),
            "low quality ({} bytes) should be smaller than high quality ({} bytes)",
            low.len(),
            high.len(),
        );
    }

    #[test]
    fn lossless_encode_at_quality_100() {
        let codec = JxlCodec;
        let original = create_test_image(32, 32);
        let options = EncodeOptions { quality: 100 };

        let encoded = codec.encode(&original, &options).expect("lossless encode failed");
        let decoded = codec.decode(&encoded).expect("lossless decode failed");

        assert_eq!(decoded.data, original.data, "lossless roundtrip must be exact");
    }
}
```

**Step 2: format.rs - can_encode()를 true로 변경**

`crates/slimg-core/src/format.rs`의 `can_encode()` 메서드에서 JXL 예외 제거:

```rust
// Before
pub fn can_encode(&self) -> bool {
    !matches!(self, Self::Jxl)
}

// After
pub fn can_encode(&self) -> bool {
    true
}
```

`can_encode()` 주석도 업데이트:

```rust
// Before
/// Whether encoding is supported for this format.
///
/// Returns `false` only for JXL due to GPL license restrictions
/// in the reference encoder.

// After
/// Whether encoding is supported for this format.
```

**Step 3: format.rs 테스트 수정**

기존 `can_encode_jxl_is_false` 테스트를 반전:

```rust
// Before
#[test]
fn can_encode_jxl_is_false() {
    assert!(!Format::Jxl.can_encode());
}

// After
#[test]
fn can_encode_jxl_is_true() {
    assert!(Format::Jxl.can_encode());
}
```

`can_encode_all_others_true` 테스트에 JXL 추가:

```rust
#[test]
fn can_encode_all_formats() {
    assert!(Format::Jpeg.can_encode());
    assert!(Format::Png.can_encode());
    assert!(Format::WebP.can_encode());
    assert!(Format::Avif.can_encode());
    assert!(Format::Jxl.can_encode());
    assert!(Format::Qoi.can_encode());
}
```

**Step 4: 테스트 실행**

```bash
cargo test -p slimg-core
```

**Step 5: 커밋**

```bash
git add crates/slimg-core/src/codec/jxl/ crates/slimg-core/src/format.rs
git commit -m "feat(slimg-core): implement JXL encoding via libjxl"
```

---

### Task 7: FFI 바인딩 업데이트 (slimg-ffi)

**Files:**
- Modify: `crates/slimg-ffi/src/lib.rs`

**Step 1: EncodingNotSupported 주석 정리**

`lib.rs`의 `SlimgError::EncodingNotSupported` variant는 그대로 유지 (다른 포맷이 향후 필요할 수 있음). 변경 사항 없음. `format_can_encode`가 이미 `slimg_core::Format::can_encode()`를 호출하므로 자동 반영.

**Step 2: 확인**

```bash
cargo build -p slimg-ffi
```

**Step 3: 커밋 (변경 사항 있는 경우에만)**

---

### Task 8: CI 빌드 설정 업데이트

**Files:**
- Modify: `dist-workspace.toml`

**Step 1: system dependencies에 cmake 추가**

```toml
[dist.dependencies.homebrew]
nasm = { stage = ["build"] }
meson = { stage = ["build"] }
ninja = { stage = ["build"] }
cmake = { stage = ["build"] }

[dist.dependencies.apt]
nasm = '*'
meson = '*'
ninja-build = '*'
cmake = '*'

[dist.dependencies.chocolatey]
nasm = '*'
ninja = '*'
cmake = '*'
```

**Step 2: 전체 빌드 + 테스트 확인**

```bash
cargo build --workspace
cargo test --workspace
```

**Step 3: 커밋**

```bash
git add dist-workspace.toml
git commit -m "ci: add cmake to system dependencies for libjxl build"
```

---

### Task 9: CLI JXL 인코딩 동작 확인

**Step 1: CLI로 JXL 인코딩 테스트**

테스트용 이미지가 있다면:

```bash
cargo run -- convert test.png -f jxl -q 80 -o test.jxl
cargo run -- convert test.jxl -f png -o roundtrip.png
```

**Step 2: 최종 확인**

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Step 3: 커밋 (clippy 수정 사항 있는 경우)**
