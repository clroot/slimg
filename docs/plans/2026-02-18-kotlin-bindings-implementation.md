# Kotlin Bindings Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** UniFFI 기반 Kotlin/JVM 바인딩을 만들어 slimg-core의 전체 공개 API를 Kotlin에서 사용할 수 있게 한다.

**Architecture:** 새로운 `slimg-ffi` crate가 slimg-core를 래핑하여 UniFFI proc macro로 FFI 레이어를 제공한다. `uniffi-bindgen`이 Kotlin 코드를 자동 생성하고, Gradle 프로젝트에서 패키징한다. slimg-core는 수정하지 않는다.

**Tech Stack:** Rust (edition 2024), UniFFI 0.31, Kotlin, Gradle (Kotlin DSL)

---

## Task 1: slimg-ffi crate 생성 및 workspace 등록

**Files:**
- Create: `crates/slimg-ffi/Cargo.toml`
- Create: `crates/slimg-ffi/src/lib.rs`
- Create: `crates/slimg-ffi/uniffi-bindgen.rs`
- Create: `crates/slimg-ffi/uniffi.toml`
- Modify: `Cargo.toml` (workspace root, line 3)

**Step 1: workspace root Cargo.toml에 slimg-ffi 추가**

`Cargo.toml` line 3을 수정:

```toml
members = ["crates/slimg-core", "crates/slimg-ffi", "cli"]
```

**Step 2: slimg-ffi/Cargo.toml 생성**

```toml
[package]
name = "slimg-ffi"
version = "0.1.2"
edition = "2024"
license = "MIT OR Apache-2.0"
description = "UniFFI bindings for slimg-core image optimization library"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
slimg-core = { path = "../slimg-core" }
uniffi = { version = "0.31", features = ["cli"] }
thiserror = "2"

[[bin]]
name = "uniffi-bindgen"
path = "uniffi-bindgen.rs"
```

**Step 3: uniffi-bindgen.rs 생성**

```rust
fn main() {
    uniffi::uniffi_bindgen_main()
}
```

**Step 4: uniffi.toml 생성**

```toml
[bindings.kotlin]
package_name = "io.clroot.slimg"
cdylib_name = "slimg_ffi"
generate_immutable_records = true
```

**Step 5: 최소 lib.rs 생성 (scaffolding만)**

```rust
uniffi::setup_scaffolding!();
```

**Step 6: 빌드 확인**

Run: `cargo build -p slimg-ffi`
Expected: 성공 (경고 있을 수 있음)

**Step 7: Commit**

```bash
git add crates/slimg-ffi/ Cargo.toml
git commit -m "feat: add slimg-ffi crate with UniFFI scaffolding"
```

---

## Task 2: Format enum UniFFI 바인딩

**Files:**
- Modify: `crates/slimg-ffi/src/lib.rs`

**Step 1: Format enum 래퍼 작성**

slimg-core의 `Format`을 UniFFI에 직접 노출할 수 없으므로 (UniFFI derive가 없음), FFI용 래퍼 enum을 만들고 변환을 구현한다.

`crates/slimg-ffi/src/lib.rs`:

```rust
uniffi::setup_scaffolding!();

use slimg_core;

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum Format {
    Jpeg,
    Png,
    WebP,
    Avif,
    Jxl,
    Qoi,
}

impl Format {
    fn to_core(self) -> slimg_core::Format {
        match self {
            Format::Jpeg => slimg_core::Format::Jpeg,
            Format::Png => slimg_core::Format::Png,
            Format::WebP => slimg_core::Format::WebP,
            Format::Avif => slimg_core::Format::Avif,
            Format::Jxl => slimg_core::Format::Jxl,
            Format::Qoi => slimg_core::Format::Qoi,
        }
    }

    fn from_core(format: slimg_core::Format) -> Self {
        match format {
            slimg_core::Format::Jpeg => Format::Jpeg,
            slimg_core::Format::Png => Format::Png,
            slimg_core::Format::WebP => Format::WebP,
            slimg_core::Format::Avif => Format::Avif,
            slimg_core::Format::Jxl => Format::Jxl,
            slimg_core::Format::Qoi => Format::Qoi,
        }
    }
}

/// Returns the canonical file extension for the given format.
#[uniffi::export]
fn format_extension(format: Format) -> String {
    format.to_core().extension().to_string()
}

/// Whether encoding is supported for the given format.
#[uniffi::export]
fn format_can_encode(format: Format) -> bool {
    format.to_core().can_encode()
}
```

**Step 2: 빌드 확인**

Run: `cargo build -p slimg-ffi`
Expected: 성공

**Step 3: Commit**

```bash
git add crates/slimg-ffi/src/lib.rs
git commit -m "feat(ffi): add Format enum with UniFFI bindings"
```

---

## Task 3: Error, ResizeMode, ImageData, Pipeline 타입 바인딩

**Files:**
- Modify: `crates/slimg-ffi/src/lib.rs`

**Step 1: Error 타입 추가**

lib.rs에 추가:

```rust
/// Errors from slimg operations.
#[derive(Debug, uniffi::Error, thiserror::Error)]
pub enum SlimgError {
    #[error("unsupported format: {format}")]
    UnsupportedFormat { format: String },

    #[error("unknown format: {detail}")]
    UnknownFormat { detail: String },

    #[error("encoding not supported: {format}")]
    EncodingNotSupported { format: String },

    #[error("decode error: {message}")]
    Decode { message: String },

    #[error("encode error: {message}")]
    Encode { message: String },

    #[error("resize error: {message}")]
    Resize { message: String },

    #[error("I/O error: {message}")]
    Io { message: String },

    #[error("image error: {message}")]
    Image { message: String },
}

impl From<slimg_core::Error> for SlimgError {
    fn from(e: slimg_core::Error) -> Self {
        match e {
            slimg_core::Error::UnsupportedFormat(f) => SlimgError::UnsupportedFormat {
                format: format!("{f:?}"),
            },
            slimg_core::Error::UnknownFormat(s) => SlimgError::UnknownFormat { detail: s },
            slimg_core::Error::EncodingNotSupported(f) => SlimgError::EncodingNotSupported {
                format: format!("{f:?}"),
            },
            slimg_core::Error::Decode(s) => SlimgError::Decode { message: s },
            slimg_core::Error::Encode(s) => SlimgError::Encode { message: s },
            slimg_core::Error::Resize(s) => SlimgError::Resize { message: s },
            slimg_core::Error::Io(e) => SlimgError::Io {
                message: e.to_string(),
            },
            slimg_core::Error::Image(e) => SlimgError::Image {
                message: e.to_string(),
            },
        }
    }
}
```

**Step 2: ResizeMode 타입 추가**

```rust
/// How to resize an image.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum ResizeMode {
    /// Set width, calculate height preserving aspect ratio.
    Width { value: u32 },
    /// Set height, calculate width preserving aspect ratio.
    Height { value: u32 },
    /// Exact dimensions (may distort the image).
    Exact { width: u32, height: u32 },
    /// Fit within bounds, preserving aspect ratio.
    Fit { max_width: u32, max_height: u32 },
    /// Scale factor (e.g. 0.5 = half size).
    Scale { factor: f64 },
}

impl ResizeMode {
    fn to_core(&self) -> slimg_core::ResizeMode {
        match self {
            ResizeMode::Width { value } => slimg_core::ResizeMode::Width(*value),
            ResizeMode::Height { value } => slimg_core::ResizeMode::Height(*value),
            ResizeMode::Exact { width, height } => slimg_core::ResizeMode::Exact(*width, *height),
            ResizeMode::Fit {
                max_width,
                max_height,
            } => slimg_core::ResizeMode::Fit(*max_width, *max_height),
            ResizeMode::Scale { factor } => slimg_core::ResizeMode::Scale(*factor),
        }
    }
}
```

**Step 3: ImageData, PipelineOptions, 결과 타입 추가**

```rust
/// Decoded image data in RGBA format (4 bytes per pixel).
#[derive(Debug, Clone, uniffi::Record)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl ImageData {
    fn to_core(&self) -> slimg_core::ImageData {
        slimg_core::ImageData::new(self.width, self.height, self.data.clone())
    }

    fn from_core(img: slimg_core::ImageData) -> Self {
        Self {
            width: img.width,
            height: img.height,
            data: img.data,
        }
    }
}

/// Options for a conversion pipeline.
#[derive(Debug, Clone, uniffi::Record)]
pub struct PipelineOptions {
    /// Target output format.
    pub format: Format,
    /// Encoding quality (0-100).
    pub quality: u8,
    /// Optional resize to apply before encoding.
    pub resize: Option<ResizeMode>,
}

/// Result of a pipeline conversion.
#[derive(Debug, Clone, uniffi::Record)]
pub struct PipelineResult {
    /// Encoded image bytes.
    pub data: Vec<u8>,
    /// Format of the encoded data.
    pub format: Format,
}

/// Result of a decode operation.
#[derive(Debug, Clone, uniffi::Record)]
pub struct DecodeResult {
    /// Decoded RGBA image data.
    pub image: ImageData,
    /// Detected format of the input.
    pub format: Format,
}
```

**Step 4: 빌드 확인**

Run: `cargo build -p slimg-ffi`
Expected: 성공

**Step 5: Commit**

```bash
git add crates/slimg-ffi/src/lib.rs
git commit -m "feat(ffi): add Error, ResizeMode, ImageData, Pipeline types"
```

---

## Task 4: 핵심 함수 바인딩 (decode, convert, optimize, output_path)

**Files:**
- Modify: `crates/slimg-ffi/src/lib.rs`

**Step 1: 함수 바인딩 추가**

lib.rs에 추가:

```rust
/// Detect the format from magic bytes and decode raw image data.
#[uniffi::export]
fn decode(data: Vec<u8>) -> Result<DecodeResult, SlimgError> {
    let (image, format) = slimg_core::decode(&data)?;
    Ok(DecodeResult {
        image: ImageData::from_core(image),
        format: Format::from_core(format),
    })
}

/// Read a file from disk, detect its format, and decode it.
#[uniffi::export]
fn decode_file(path: String) -> Result<DecodeResult, SlimgError> {
    let (image, format) = slimg_core::decode_file(std::path::Path::new(&path))?;
    Ok(DecodeResult {
        image: ImageData::from_core(image),
        format: Format::from_core(format),
    })
}

/// Convert an image to the specified format, optionally resizing first.
#[uniffi::export]
fn convert(image: &ImageData, options: &PipelineOptions) -> Result<PipelineResult, SlimgError> {
    let core_options = slimg_core::PipelineOptions {
        format: options.format.to_core(),
        quality: options.quality,
        resize: options.resize.as_ref().map(|r| r.to_core()),
    };
    let result = slimg_core::convert(&image.to_core(), &core_options)?;
    Ok(PipelineResult {
        data: result.data,
        format: Format::from_core(result.format),
    })
}

/// Decode the data and re-encode in the same format at the given quality.
#[uniffi::export]
fn optimize(data: Vec<u8>, quality: u8) -> Result<PipelineResult, SlimgError> {
    let result = slimg_core::optimize(&data, quality)?;
    Ok(PipelineResult {
        data: result.data,
        format: Format::from_core(result.format),
    })
}

/// Derive an output path for the converted image.
///
/// - If `output` is `None`, uses the input directory with the new extension.
/// - If `output` is a directory, places the file there with the new extension.
/// - Otherwise, uses `output` as-is.
#[uniffi::export]
fn output_path(input: String, format: Format, output: Option<String>) -> String {
    let result = slimg_core::output_path(
        std::path::Path::new(&input),
        format.to_core(),
        output.as_ref().map(|s| std::path::Path::new(s.as_str())),
    );
    result.to_string_lossy().to_string()
}

/// Detect format from file extension (case-insensitive).
#[uniffi::export]
fn format_from_extension(path: String) -> Option<Format> {
    slimg_core::Format::from_extension(std::path::Path::new(&path)).map(Format::from_core)
}

/// Detect format from magic bytes at the start of file data.
#[uniffi::export]
fn format_from_magic_bytes(data: Vec<u8>) -> Option<Format> {
    slimg_core::Format::from_magic_bytes(&data).map(Format::from_core)
}
```

**Step 2: 빌드 확인**

Run: `cargo build -p slimg-ffi`
Expected: 성공

**Step 3: Commit**

```bash
git add crates/slimg-ffi/src/lib.rs
git commit -m "feat(ffi): add decode, convert, optimize, output_path functions"
```

---

## Task 5: Kotlin 바인딩 생성 및 검증

**Files:**
- Generated: `bindings/kotlin/` (uniffi-bindgen 출력)

**Step 1: 릴리즈 빌드**

Run: `cargo build --release -p slimg-ffi`
Expected: 성공

**Step 2: Kotlin 바인딩 생성**

macOS의 경우:

```bash
cargo run -p slimg-ffi --bin uniffi-bindgen generate \
    --library target/release/libslimg_ffi.dylib \
    --language kotlin \
    --out-dir bindings/kotlin/src/main/kotlin
```

Linux의 경우:

```bash
cargo run -p slimg-ffi --bin uniffi-bindgen generate \
    --library target/release/libslimg_ffi.so \
    --language kotlin \
    --out-dir bindings/kotlin/src/main/kotlin
```

Expected: `bindings/kotlin/src/main/kotlin/io/clroot/slimg/slimg_ffi.kt` 생성됨

**Step 3: 생성된 Kotlin 코드 확인**

생성된 파일을 열어 다음을 확인:
- `enum class Format` (fieldless enum)
- `sealed class ResizeMode` (data-carrying enum → sealed class)
- `data class ImageData`, `PipelineOptions`, `PipelineResult`, `DecodeResult`
- `sealed class SlimgException` (에러 타입)
- Top-level 함수: `decode()`, `decodeFile()`, `convert()`, `optimize()`, `outputPath()`

**Step 4: Commit**

```bash
git add bindings/kotlin/src/main/kotlin/
git commit -m "feat(ffi): generate Kotlin bindings via UniFFI"
```

---

## Task 6: Gradle 프로젝트 설정

**Files:**
- Create: `bindings/kotlin/build.gradle.kts`
- Create: `bindings/kotlin/settings.gradle.kts`
- Create: `bindings/kotlin/gradle.properties`

**Step 1: settings.gradle.kts 생성**

```kotlin
rootProject.name = "slimg-kotlin"
```

**Step 2: gradle.properties 생성**

```properties
group=io.clroot.slimg
version=0.1.2
kotlin.code.style=official
```

**Step 3: build.gradle.kts 생성**

```kotlin
plugins {
    kotlin("jvm") version "2.1.0"
    `maven-publish`
    signing
}

group = property("group") as String
version = property("version") as String

repositories {
    mavenCentral()
}

dependencies {
    implementation("net.java.dev.jna:jna:5.16.0")
    testImplementation(kotlin("test"))
}

kotlin {
    jvmToolchain(17)
}

// Include native libraries from resources
sourceSets {
    main {
        resources.srcDirs("src/main/resources")
    }
}

tasks.test {
    useJUnitPlatform()
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])

            pom {
                name.set("slimg-kotlin")
                description.set("Kotlin bindings for slimg image optimization library")
                url.set("https://github.com/clroot/slimg")

                licenses {
                    license {
                        name.set("MIT OR Apache-2.0")
                        url.set("https://github.com/clroot/slimg/blob/main/LICENSE")
                    }
                }

                developers {
                    developer {
                        id.set("clroot")
                        name.set("clroot")
                    }
                }

                scm {
                    connection.set("scm:git:git://github.com/clroot/slimg.git")
                    developerConnection.set("scm:git:ssh://github.com/clroot/slimg.git")
                    url.set("https://github.com/clroot/slimg")
                }
            }
        }
    }
}
```

**Step 4: Commit**

```bash
git add bindings/kotlin/
git commit -m "feat(kotlin): add Gradle project for Kotlin bindings"
```

---

## Task 7: 네이티브 라이브러리 번들링 및 로컬 테스트

**Files:**
- Create: `bindings/kotlin/src/main/resources/` (플랫폼별 디렉토리)
- Create: `bindings/kotlin/src/test/kotlin/io/clroot/slimg/SlimgTest.kt`

**Step 1: 네이티브 라이브러리를 resources에 복사 (로컬 개발용)**

macOS Apple Silicon의 경우:

```bash
mkdir -p bindings/kotlin/src/main/resources/darwin-aarch64
cp target/release/libslimg_ffi.dylib bindings/kotlin/src/main/resources/darwin-aarch64/
```

macOS Intel의 경우:

```bash
mkdir -p bindings/kotlin/src/main/resources/darwin-x86-64
cp target/release/libslimg_ffi.dylib bindings/kotlin/src/main/resources/darwin-x86-64/
```

Linux x86_64의 경우:

```bash
mkdir -p bindings/kotlin/src/main/resources/linux-x86-64
cp target/release/libslimg_ffi.so bindings/kotlin/src/main/resources/linux-x86-64/
```

**Step 2: Kotlin 테스트 작성**

`bindings/kotlin/src/test/kotlin/io/clroot/slimg/SlimgTest.kt`:

```kotlin
package io.clroot.slimg

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import kotlin.test.assertNull
import kotlin.test.assertFailsWith

class SlimgTest {

    @Test
    fun `formatExtension returns correct extension`() {
        assertEquals("jpg", formatExtension(Format.JPEG))
        assertEquals("png", formatExtension(Format.PNG))
        assertEquals("webp", formatExtension(Format.WEBP))
        assertEquals("avif", formatExtension(Format.AVIF))
        assertEquals("jxl", formatExtension(Format.JXL))
        assertEquals("qoi", formatExtension(Format.QOI))
    }

    @Test
    fun `formatCanEncode returns false for JXL`() {
        assertTrue(formatCanEncode(Format.JPEG))
        assertTrue(formatCanEncode(Format.PNG))
        assertTrue(formatCanEncode(Format.WEBP))
        kotlin.test.assertFalse(formatCanEncode(Format.JXL))
    }

    @Test
    fun `formatFromExtension detects known formats`() {
        assertEquals(Format.JPEG, formatFromExtension("photo.jpg"))
        assertEquals(Format.PNG, formatFromExtension("image.png"))
        assertEquals(Format.WEBP, formatFromExtension("image.webp"))
    }

    @Test
    fun `formatFromExtension returns null for unknown`() {
        assertNull(formatFromExtension("file.bmp"))
    }

    @Test
    fun `formatFromMagicBytes detects JPEG`() {
        val jpegHeader = byteArrayOf(0xFF.toByte(), 0xD8.toByte(), 0xFF.toByte(), 0xE0.toByte())
        assertEquals(Format.JPEG, formatFromMagicBytes(jpegHeader))
    }

    @Test
    fun `formatFromMagicBytes detects PNG`() {
        val pngHeader = byteArrayOf(0x89.toByte(), 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A)
        assertEquals(Format.PNG, formatFromMagicBytes(pngHeader))
    }

    @Test
    fun `formatFromMagicBytes returns null for unknown`() {
        assertNull(formatFromMagicBytes(byteArrayOf(0x00, 0x00, 0x00, 0x00)))
    }

    @Test
    fun `outputPath changes extension`() {
        val result = outputPath("/tmp/photo.jpg", Format.WEBP, null)
        assertEquals("/tmp/photo.webp", result)
    }

    @Test
    fun `outputPath with explicit output`() {
        val result = outputPath("/tmp/photo.jpg", Format.PNG, "/out/result.png")
        assertEquals("/out/result.png", result)
    }

    @Test
    fun `decode rejects unknown format`() {
        assertFailsWith<SlimgException.UnknownFormat> {
            decode(byteArrayOf(0x00, 0x00, 0x00, 0x00))
        }
    }
}
```

**Step 3: 테스트 실행**

```bash
cd bindings/kotlin && ./gradlew test
```

Expected: 모든 테스트 통과

**Step 4: Commit**

```bash
git add bindings/kotlin/
git commit -m "feat(kotlin): add native library bundling and integration tests"
```

---

## Task 8: .gitignore 및 정리

**Files:**
- Create: `bindings/kotlin/.gitignore`
- Modify: `.gitignore` (root, if exists)

**Step 1: Kotlin 프로젝트 .gitignore 생성**

`bindings/kotlin/.gitignore`:

```
.gradle/
build/
*.class
local.properties

# Native libraries are built, not committed
src/main/resources/darwin-*/
src/main/resources/linux-*/
src/main/resources/win32-*/
```

**Step 2: 루트 .gitignore에 추가 (있다면)**

UniFFI 생성 코드는 커밋할지 여부를 결정해야 한다. 생성 코드를 커밋하면 Rust 빌드 없이도 Kotlin 프로젝트를 사용할 수 있고, 커밋하지 않으면 항상 최신 상태가 보장된다.

**권장: 생성 코드는 커밋한다** (CI에서 검증 가능하도록).

**Step 3: Commit**

```bash
git add bindings/kotlin/.gitignore
git commit -m "chore(kotlin): add .gitignore for Kotlin bindings"
```

---

## Task 9: CI 워크플로우 (빌드 및 테스트)

**Files:**
- Create: `.github/workflows/kotlin-bindings.yml`

**Step 1: GitHub Actions 워크플로우 생성**

`.github/workflows/kotlin-bindings.yml`:

```yaml
name: Kotlin Bindings

on:
  push:
    branches: [main]
    paths:
      - 'crates/slimg-ffi/**'
      - 'crates/slimg-core/**'
      - 'bindings/kotlin/**'
  pull_request:
    paths:
      - 'crates/slimg-ffi/**'
      - 'crates/slimg-core/**'
      - 'bindings/kotlin/**'

jobs:
  build-and-test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies (macOS)
        if: runner.os == 'macOS'
        run: brew install nasm dav1d

      - name: Install system dependencies (Linux)
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y nasm

      - name: Build slimg-ffi
        run: cargo build --release -p slimg-ffi

      - name: Generate Kotlin bindings
        run: |
          if [[ "$OSTYPE" == "darwin"* ]]; then
            LIB_EXT="dylib"
          else
            LIB_EXT="so"
          fi
          cargo run -p slimg-ffi --bin uniffi-bindgen generate \
            --library target/release/libslimg_ffi.$LIB_EXT \
            --language kotlin \
            --out-dir bindings/kotlin/src/main/kotlin

      - name: Copy native library
        run: |
          if [[ "$OSTYPE" == "darwin"* ]]; then
            ARCH=$(uname -m)
            if [[ "$ARCH" == "arm64" ]]; then
              DIR="darwin-aarch64"
            else
              DIR="darwin-x86-64"
            fi
            mkdir -p bindings/kotlin/src/main/resources/$DIR
            cp target/release/libslimg_ffi.dylib bindings/kotlin/src/main/resources/$DIR/
          else
            mkdir -p bindings/kotlin/src/main/resources/linux-x86-64
            cp target/release/libslimg_ffi.so bindings/kotlin/src/main/resources/linux-x86-64/
          fi

      - name: Set up JDK 17
        uses: actions/setup-java@v4
        with:
          java-version: '17'
          distribution: 'temurin'

      - name: Setup Gradle
        uses: gradle/actions/setup-gradle@v4

      - name: Run Kotlin tests
        working-directory: bindings/kotlin
        run: ./gradlew test
```

**Step 2: Commit**

```bash
git add .github/workflows/kotlin-bindings.yml
git commit -m "ci: add Kotlin bindings build and test workflow"
```

---

## Summary

| Task | Description | Depends On |
|------|-------------|------------|
| 1 | slimg-ffi crate 생성 및 workspace 등록 | - |
| 2 | Format enum UniFFI 바인딩 | 1 |
| 3 | Error, ResizeMode, ImageData, Pipeline 타입 | 2 |
| 4 | 핵심 함수 바인딩 (decode, convert, optimize) | 3 |
| 5 | Kotlin 바인딩 생성 및 검증 | 4 |
| 6 | Gradle 프로젝트 설정 | 5 |
| 7 | 네이티브 라이브러리 번들링 및 로컬 테스트 | 6 |
| 8 | .gitignore 및 정리 | 7 |
| 9 | CI 워크플로우 | 8 |
