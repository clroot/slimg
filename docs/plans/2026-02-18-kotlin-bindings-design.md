# slimg Kotlin Bindings Design

## Overview

slimg-core의 전체 공개 API를 Kotlin/JVM 및 Android에서 사용할 수 있도록 UniFFI 기반 바인딩을 제공한다.

## Goals

- Android 앱과 JVM 서버(Spring 등) 양쪽에서 사용 가능
- slimg-core의 전체 공개 API(decode, convert, optimize, resize) 노출
- Maven Central 배포 (`io.clroot.slimg:slimg-kotlin`)
- slimg-core는 수정하지 않음

## Approach: UniFFI (Mozilla)

UniFFI proc macro 방식으로 Rust FFI 레이어를 정의하고, Kotlin 바인딩 코드를 자동 생성한다.

선택 이유:
- 타입 매핑 자동화 (enum, struct, error)
- 향후 Swift/Python 등 다른 언어 바인딩 추가 용이
- Android NDK 호환 검증된 도구

## Project Structure

```
slimg/
├── crates/
│   ├── slimg-core/          # (기존, 변경 없음)
│   └── slimg-ffi/           # (신규) UniFFI 바인딩 crate
│       ├── Cargo.toml
│       ├── src/
│       │   └── lib.rs       # UniFFI 래퍼 + proc macro
│       └── uniffi-bindgen.rs # bindgen CLI 진입점
│
└── bindings/
    └── kotlin/              # 생성된 Kotlin 코드 + Gradle 프로젝트
        ├── build.gradle.kts
        ├── src/main/kotlin/io/clroot/slimg/
        └── src/test/kotlin/
```

## Type Mapping

### Enums

| Rust | Kotlin |
|------|--------|
| `Format { Jpeg, Png, WebP, Avif, Jxl, Qoi }` | `enum class Format { JPEG, PNG, WEBP, AVIF, JXL, QOI }` |
| `ResizeMode::Width(u32)` | `sealed class ResizeMode` → `data class Width(val value: UInt)` |
| `ResizeMode::Height(u32)` | `data class Height(val value: UInt)` |
| `ResizeMode::Exact(u32, u32)` | `data class Exact(val width: UInt, val height: UInt)` |
| `ResizeMode::Fit(u32, u32)` | `data class Fit(val maxWidth: UInt, val maxHeight: UInt)` |
| `ResizeMode::Scale(f64)` | `data class Scale(val factor: Double)` |

### Structs

| Rust | Kotlin |
|------|--------|
| `ImageData { width: u32, height: u32, data: Vec<u8> }` | `data class ImageData(width: UInt, height: UInt, data: ByteArray)` |
| `PipelineOptions { format, quality: u8, resize: Option<ResizeMode> }` | `data class PipelineOptions(format: Format, quality: UByte, resize: ResizeMode?)` |
| `PipelineResult { data: Vec<u8>, format }` | `data class PipelineResult(data: ByteArray, format: Format)` |
| (신규) `DecodeResult` | `data class DecodeResult(image: ImageData, format: Format)` |

### Functions

| Rust | Kotlin |
|------|--------|
| `decode(&[u8]) -> Result<(ImageData, Format)>` | `Slimg.decode(data: ByteArray): DecodeResult` |
| `decode_file(&Path) -> Result<(ImageData, Format)>` | `Slimg.decodeFile(path: String): DecodeResult` |
| `convert(&ImageData, &PipelineOptions) -> Result<PipelineResult>` | `Slimg.convert(image: ImageData, options: PipelineOptions): PipelineResult` |
| `optimize(&[u8], u8) -> Result<PipelineResult>` | `Slimg.optimize(data: ByteArray, quality: UByte): PipelineResult` |
| `output_path(&Path, Format, Option<&Path>) -> PathBuf` | `Slimg.outputPath(input: String, format: Format, output: String?): String` |

### Errors

| Rust Error variant | Kotlin Exception |
|-------------------|-----------------|
| `UnsupportedFormat(Format)` | `SlimgException.UnsupportedFormat(format)` |
| `UnknownFormat(String)` | `SlimgException.UnknownFormat(filename)` |
| `EncodingNotSupported(Format)` | `SlimgException.EncodingNotSupported(format)` |
| `Decode(String)` | `SlimgException.Decode(message)` |
| `Encode(String)` | `SlimgException.Encode(message)` |
| `Resize(String)` | `SlimgException.Resize(message)` |
| `Io(io::Error)` | `SlimgException.Io(message)` |
| `Image(ImageError)` | `SlimgException.Image(message)` |

## Build & Distribution

### Cross-compilation Targets

| Platform | Rust Target | Library | JAR Path |
|----------|-------------|---------|----------|
| macOS (Apple Silicon) | `aarch64-apple-darwin` | `libslimg_ffi.dylib` | `darwin-aarch64/` |
| macOS (Intel) | `x86_64-apple-darwin` | `libslimg_ffi.dylib` | `darwin-x86-64/` |
| Linux (x86_64) | `x86_64-unknown-linux-gnu` | `libslimg_ffi.so` | `linux-x86-64/` |
| Linux (aarch64) | `aarch64-unknown-linux-gnu` | `libslimg_ffi.so` | `linux-aarch64/` |
| Windows (x86_64) | `x86_64-pc-windows-msvc` | `slimg_ffi.dll` | `win32-x86-64/` |
| Android (arm64) | `aarch64-linux-android` | `libslimg_ffi.so` | `jniLibs/arm64-v8a/` |
| Android (x86_64) | `x86_64-linux-android` | `libslimg_ffi.so` | `jniLibs/x86_64/` |

### Build Pipeline

1. `cargo build --release -p slimg-ffi --target <TARGET>` (각 타겟별)
2. `cargo run -p slimg-ffi --bin uniffi-bindgen generate` (Kotlin 소스 생성)
3. Gradle: Kotlin 코드 + 네이티브 라이브러리 → JAR 패키징
4. Maven Central 배포 (signing + publishing)

### Maven Coordinates

```kotlin
implementation("io.clroot.slimg:slimg-kotlin:0.1.2")
```

### CI/CD (GitHub Actions)

```
trigger: tag v*.*.*
jobs:
  build-native:
    matrix: [macOS-arm64, macOS-x64, linux-x64, linux-arm64, windows-x64, android-arm64, android-x64]
    steps: cargo build --release --target $TARGET

  generate-bindings:
    steps: uniffi-bindgen generate → Kotlin source

  publish:
    needs: [build-native, generate-bindings]
    steps: collect native libs → Gradle build → Maven Central publish
```

## Kotlin API Design

### Usage Example

```kotlin
import io.clroot.slimg.*

// Decode from file
val result = Slimg.decodeFile("/path/to/photo.jpg")

// Convert to WebP with resize
val webp = Slimg.convert(
    image = result.image,
    options = PipelineOptions(
        format = Format.WEBP,
        quality = 80u,
        resize = ResizeMode.Width(800u)
    )
)

// Save result
webp.save("/path/to/output.webp")

// Optimize in-memory bytes
val optimized = Slimg.optimize(imageBytes, quality = 75u)

// Auto-generate output path
val outputPath = Slimg.outputPath("/photos/pic.png", Format.WEBP)
```

### Error Handling

```kotlin
try {
    val result = Slimg.decodeFile("/path/to/corrupted.xyz")
} catch (e: SlimgException.UnknownFormat) {
    println("Unknown format: ${e.filename}")
} catch (e: SlimgException) {
    println("Error: ${e.message}")
}
```

### Package Structure

```
io.clroot.slimg/
├── Slimg.kt              # Top-level functions
├── Format.kt             # enum class Format
├── ResizeMode.kt         # sealed class ResizeMode
├── ImageData.kt          # data class ImageData
├── PipelineOptions.kt    # data class PipelineOptions
├── PipelineResult.kt     # data class PipelineResult + save()
├── DecodeResult.kt       # data class DecodeResult(image, format)
└── SlimgException.kt     # sealed class SlimgException
```

## Android Compatibility

Android에서는 `ContentResolver`/`InputStream` 사용이 일반적이므로, 바이트 배열 기반 API(`decode`, `optimize`)가 핵심. `decodeFile`은 파일 시스템 접근 가능 시에만 사용.
