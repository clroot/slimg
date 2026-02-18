# slimg-kotlin

Kotlin/JVM bindings for the [slimg](https://github.com/clroot/slimg) image optimization library.

Supports macOS (Apple Silicon, Intel), Linux (x86_64, ARM64), and Windows (x86_64) — all native libraries are bundled in a single JAR.

## Installation

### Gradle (Kotlin DSL)

```kotlin
dependencies {
    implementation("io.clroot.slimg:slimg-kotlin:0.3.1")
}
```

### Gradle (Groovy)

```groovy
dependencies {
    implementation 'io.clroot.slimg:slimg-kotlin:0.3.1'
}
```

### Maven

```xml
<dependency>
    <groupId>io.clroot.slimg</groupId>
    <artifactId>slimg-kotlin</artifactId>
    <version>0.3.1</version>
</dependency>
```

## Usage

```kotlin
import io.clroot.slimg.*

// Decode an image file
val result = decodeFile("photo.jpg")
println("${result.image.width}x${result.image.height} ${result.format}")

// Convert to WebP
val webp = convert(result.image, PipelineOptions(
    format = Format.WEB_P,
    quality = 80u,
    resize = null,
))
File("photo.webp").writeBytes(webp.data)

// Optimize in the same format
val fileBytes = File("photo.png").readBytes()
val optimized = optimize(fileBytes, 80u)
File("photo-optimized.png").writeBytes(optimized.data)

// Resize and convert
val resized = convert(result.image, PipelineOptions(
    format = Format.WEB_P,
    quality = 80u,
    resize = ResizeMode.Width(800u),
))
```

## API Reference

### Functions

| Function | Description |
|----------|-------------|
| `decode(data: ByteArray)` | Decode image from bytes |
| `decodeFile(path: String)` | Decode image from file path |
| `convert(image, options)` | Convert image to a different format |
| `crop(image, mode)` | Crop an image by region or aspect ratio |
| `extend(image, mode, fill)` | Extend (pad) image canvas |
| `resize(image, mode)` | Resize an image |
| `optimize(data: ByteArray, quality: UByte)` | Re-encode to reduce file size |
| `outputPath(input, format, output?)` | Generate output file path |
| `formatExtension(format)` | Get file extension for a format |
| `formatCanEncode(format)` | Check if format supports encoding |
| `formatFromExtension(path)` | Detect format from file extension |
| `formatFromMagicBytes(data)` | Detect format from file header |

### Types

| Type | Description |
|------|-------------|
| `Format` | `JPEG`, `PNG`, `WEB_P`, `AVIF`, `JXL`, `QOI` |
| `ResizeMode` | `Width`, `Height`, `Exact`, `Fit`, `Scale` |
| `CropMode` | `Region`, `AspectRatio` |
| `ExtendMode` | `AspectRatio`, `Size` |
| `FillColor` | `Transparent`, `Solid(r, g, b, a)` |
| `PipelineOptions` | `format`, `quality`, `resize`, `crop`, `extend`, `fillColor` |
| `PipelineResult` | `data` (ByteArray), `format` |
| `DecodeResult` | `image` (ImageData), `format` |
| `ImageData` | `width`, `height`, `data` (raw pixels) |
| `SlimgException` | Error with subclasses: `UnsupportedFormat`, `UnknownFormat`, `EncodingNotSupported`, `Decode`, `Encode`, `Resize`, `Crop`, `Extend`, `Io`, `Image` |

## Supported Platforms

| Platform | Architecture | Status |
|----------|-------------|--------|
| macOS | Apple Silicon (aarch64) | Supported |
| macOS | Intel (x86_64) | Supported |
| Linux | x86_64 | Supported |
| Linux | ARM64 (aarch64) | Supported |
| Windows | x86_64 | Supported |

## Requirements

- JDK 17+
- JNA 5.16.0+ (transitive dependency)

## License

MIT OR Apache-2.0
