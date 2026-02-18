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

## Quick Start

```kotlin
import io.clroot.slimg.*

// Decode → transform → encode in a single chain
val result = SlimgImage.decodeFile("photo.jpg")
    .resize(width = 800)
    .crop(aspectRatio = 16 to 9)
    .encode(Format.WEB_P, quality = 85)

File("photo.webp").writeBytes(result.data)
```

## Usage

### SlimgImage (Fluent API)

Chain decode → transform → encode operations fluently. All parameters use `Int` — no `UInt`/`UByte` conversions needed.

```kotlin
// Decode from various sources
val img = SlimgImage.decode(byteArray)
val img = SlimgImage.decode(inputStream)
val img = SlimgImage.decode(byteBuffer)
val img = SlimgImage.decodeFile("photo.jpg")

// Resize
img.resize(width = 800)                  // by width (preserve aspect ratio)
img.resize(height = 600)                 // by height
img.resize(width = 800, height = 600)    // exact dimensions
img.resize(fit = 1200 to 1200)           // fit within bounds
img.resize(scale = 0.5)                  // scale factor

// Crop
img.crop(aspectRatio = 16 to 9)          // center-anchored
img.crop(x = 100, y = 50, width = 800, height = 600)  // region

// Extend (add padding)
img.extend(1920, 1080)                   // to exact size
img.extend(aspectRatio = 1 to 1)         // to aspect ratio
img.extend(1920, 1080, fill = Slimg.solidColor(255, 0, 0))  // with color

// Encode
img.encode(Format.WEB_P, quality = 85)   // to specific format
img.optimize(quality = 70)               // re-encode in source format
```

#### Chaining

Each operation returns a new `SlimgImage`, so you can chain freely:

```kotlin
val result = SlimgImage.decodeFile("photo.jpg")
    .resize(fit = 1200 to 1200)
    .crop(aspectRatio = 1 to 1)
    .extend(aspectRatio = 16 to 9, fill = Slimg.solidColor(0, 0, 0))
    .encode(Format.WEB_P, quality = 80)

File("output.webp").writeBytes(result.data)
```

#### With InputStream / ByteBuffer

Works with any byte source — file streams, network responses, WebFlux `DataBuffer`, etc.

```kotlin
// From InputStream
FileInputStream("photo.jpg").use { stream ->
    val result = SlimgImage.decode(stream)
        .resize(width = 800)
        .encode(Format.WEB_P)
    File("photo.webp").writeBytes(result.data)
}

// From ByteBuffer (e.g. WebFlux DataBuffer)
val result = SlimgImage.decode(dataBuffer.asByteBuffer())
    .resize(fit = 1200 to 1200)
    .encode(Format.WEB_P, quality = 85)
```

### Slimg Object

For one-off operations without chaining:

```kotlin
// Decode
val decoded = Slimg.decode(byteArray)        // or InputStream, ByteBuffer
val decoded = Slimg.decodeFile("photo.jpg")

// Convert with simplified parameters
val result = Slimg.convert(
    decoded.image,
    Format.WEB_P,
    quality = 85,                           // Int, not UByte
    resize = ResizeMode.Width(800u),
    crop = CropMode.AspectRatio(16u, 9u),
)

// Optimize
val optimized = Slimg.optimize(fileBytes, quality = 70)  // Int quality

// Image operations
val resized = Slimg.resize(image, ResizeMode.Width(800u))
val cropped = Slimg.crop(image, CropMode.AspectRatio(16u, 9u))
val extended = Slimg.extend(image, ExtendMode.Size(1920u, 1080u))

// FillColor helper
val red = Slimg.solidColor(255, 0, 0)       // Int RGBA, alpha defaults to 255
```

### Low-level Functions

Direct FFI bindings are also available as top-level functions:

```kotlin
val decoded = decode(byteArray)
val result = convert(image, PipelineOptions(Format.WEB_P, 80u.toUByte(), null, null, null, null))
val optimized = optimize(byteArray, 80u.toUByte())
```

## API Layers

| Layer | Use When | Example |
|-------|----------|---------|
| `SlimgImage` | Chaining multiple operations | `SlimgImage.decode(bytes).resize(width = 800).encode(Format.WEB_P)` |
| `Slimg` object | One-off operations with `Int` params | `Slimg.optimize(bytes, quality = 80)` |
| Top-level functions | Direct FFI access needed | `decode(bytes)`, `convert(image, options)` |

## API Reference

### SlimgImage

| Method | Description |
|--------|-------------|
| `SlimgImage.decode(ByteArray / InputStream / ByteBuffer)` | Decode from bytes |
| `SlimgImage.decodeFile(path)` | Decode from file path |
| `SlimgImage.from(ImageData, format?)` | Wrap existing ImageData |
| `.resize(width?, height?)` | Resize by width/height/exact |
| `.resize(fit: Pair)` | Fit within bounds |
| `.resize(scale: Double)` | Scale by factor |
| `.crop(x, y, width, height)` | Crop region |
| `.crop(aspectRatio: Pair)` | Crop to aspect ratio |
| `.extend(width, height, fill?)` | Extend to exact size |
| `.extend(aspectRatio: Pair, fill?)` | Extend to aspect ratio |
| `.encode(format, quality?)` | Encode to format |
| `.optimize(quality?)` | Re-encode in source format |
| `.width` / `.height` | Current dimensions |
| `.format` | Source format (from decode) |
| `.imageData` | Underlying ImageData |

### Types

| Type | Description |
|------|-------------|
| `Format` | `JPEG`, `PNG`, `WEB_P`, `AVIF`, `JXL`, `QOI` |
| `ResizeMode` | `Width`, `Height`, `Exact`, `Fit`, `Scale` |
| `CropMode` | `Region`, `AspectRatio` |
| `ExtendMode` | `AspectRatio`, `Size` |
| `FillColor` | `Transparent`, `Solid(r, g, b, a)` |
| `PipelineResult` | `data` (ByteArray), `format` |
| `DecodeResult` | `image` (ImageData), `format` |
| `ImageData` | `width`, `height`, `data` (raw RGBA pixels) |
| `SlimgException` | `UnsupportedFormat`, `UnknownFormat`, `EncodingNotSupported`, `Decode`, `Encode`, `Resize`, `Crop`, `Extend`, `Io`, `Image` |

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
