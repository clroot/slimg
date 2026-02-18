# Native Library Bundling Design

## Overview

Kotlin 바인딩 JAR에 5개 플랫폼의 네이티브 라이브러리를 번들링하여, 사용자가 단일 의존성으로 모든 플랫폼에서 사용할 수 있게 한다.

## Strategy: Single Fat JAR

sqlite-jdbc, netty-tcnative 등 대부분의 JVM 네이티브 라이브러리가 사용하는 방식.

## Target Platforms

| Platform | Rust Target | Library File | JAR Resource Path |
|----------|-------------|-------------|-------------------|
| macOS Apple Silicon | `aarch64-apple-darwin` | `libslimg_ffi.dylib` | `darwin-aarch64/` |
| macOS Intel | `x86_64-apple-darwin` | `libslimg_ffi.dylib` | `darwin-x86-64/` |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `libslimg_ffi.so` | `linux-x86-64/` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `libslimg_ffi.so` | `linux-aarch64/` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `slimg_ffi.dll` | `win32-x86-64/` |

## CI Workflow

```
release tag (v*.*.*)
  │
  ├─ Job 1: build-native (matrix: 5 platforms)
  │   ├─ macOS runner → aarch64-apple-darwin, x86_64-apple-darwin
  │   ├─ Linux runner  → x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu (cross)
  │   └─ Windows runner → x86_64-pc-windows-msvc
  │   각각 cargo build --release → artifact upload
  │
  ├─ Job 2: generate-bindings
  │   └─ uniffi-bindgen → Kotlin source
  │
  └─ Job 3: package-and-publish (needs: build-native, generate-bindings)
      ├─ Download all native libraries
      ├─ Place in resources/ directories
      ├─ Gradle build → fat JAR
      └─ Maven Central publish (signing + publishing)
```

## Workflow Files

- `kotlin-bindings.yml` (existing): PR/push build + test (unchanged)
- `kotlin-release.yml` (new): release tag → cross build + package + Maven publish

## Build Dependencies

- macOS: `nasm`, `dav1d` (via Homebrew)
- Linux: `nasm` (via apt), cross-compilation toolchain for aarch64
- Windows: `nasm` (via Chocolatey), added to PATH via `build-setup.yml`
