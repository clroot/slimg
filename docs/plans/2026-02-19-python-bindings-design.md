# Python Bindings Design

## Overview

slimg에 Python 바인딩을 추가한다. 기존 UniFFI 기반 FFI 레이어(`slimg-ffi`)를 재사용하고, maturin으로 wheel을 빌드하여 PyPI에 배포한다.

- **패키지명**: `slimg` (`pip install slimg`)
- **Python 지원 범위**: 3.9 ~ 3.13
- **플랫폼**: macOS(arm64, x86_64), Linux(x86_64, aarch64), Windows(x86_64)
- **API 스타일**: UniFFI 자동생성 + Pythonic 래퍼

## Project Structure

```
bindings/python/
├── pyproject.toml              # maturin 빌드 설정 (bindings = "uniffi")
├── slimg/
│   ├── __init__.py             # Pythonic public API
│   ├── _types.py               # type alias, Enum re-export
│   └── py.typed                # PEP 561 type stub marker
├── tests/
│   ├── conftest.py             # 테스트 픽스처 (샘플 이미지 등)
│   └── test_slimg.py           # pytest 기반 테스트
└── README.md
```

## Build System

maturin `bindings = "uniffi"` 모드로 기존 `slimg-ffi` 크레이트를 직접 빌드한다.

```toml
# bindings/python/pyproject.toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "slimg"
requires-python = ">=3.9"
description = "Fast image optimization library powered by Rust"
license = { text = "MIT OR Apache-2.0" }
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
]

[tool.maturin]
manifest-path = "../../crates/slimg-ffi/Cargo.toml"
bindings = "uniffi"
python-source = "."
module-name = "slimg._lowlevel"
```

### uniffi.toml 변경

기존 Kotlin 설정에 Python 설정을 추가한다:

```toml
[bindings.kotlin]
package_name = "io.clroot.slimg"
cdylib_name = "slimg_ffi"
generate_immutable_records = true

[bindings.python]
cdylib_name = "slimg_ffi"
```

## Pythonic Wrapper API

UniFFI 자동생성 코드(`_lowlevel`)를 내부 모듈로 격리하고, `__init__.py`에서 Pythonic한 API를 제공한다.

### Public API

```python
import slimg

# -- Decoding --
image = slimg.open("photo.jpg")             # 파일에서 디코딩
image = slimg.decode(raw_bytes)             # bytes에서 디코딩

image.width    # u32
image.height   # u32
image.format   # slimg.Format.JPEG
image.data     # bytes (RGBA)

# -- Convert --
result = slimg.convert(image, format="webp", quality=80)
result = slimg.convert(
    image,
    format="avif",
    quality=60,
    resize=slimg.Resize.fit(1920, 1080),
)

result.data    # bytes (인코딩된 이미지)
result.format  # slimg.Format.WEBP
result.save("output.webp")

# -- Image operations --
cropped = slimg.crop(image, region=(10, 20, 100, 80))
cropped = slimg.crop(image, aspect_ratio=(16, 9))

extended = slimg.extend(image, aspect_ratio=(1, 1), fill="transparent")
extended = slimg.extend(image, size=(1920, 1080), fill=(255, 255, 255))

resized = slimg.resize(image, width=800)
resized = slimg.resize(image, scale=0.5)
resized = slimg.resize(image, fit=(1920, 1080))

# -- Optimize (re-encode) --
result = slimg.optimize(raw_bytes, quality=80)
result = slimg.optimize_file("photo.jpg", quality=80)

# -- Format utilities --
slimg.Format.JPEG.extension      # "jpg"
slimg.Format.JPEG.can_encode     # True
slimg.Format.JXL.can_encode      # False
slimg.Format.from_path("a.webp") # Format.WEBP
```

### Wrapper Design Principles

| Principle | Application |
|-----------|-------------|
| String to Enum | `format="webp"` -> `Format.WEB_P` |
| Remove None defaults | keyword args only, no `None, None, None` |
| Tuple to Object | `region=(10, 20, 100, 80)` -> `CropMode.Region(...)` |
| Convenience methods | `result.save()`, `slimg.open()`, `slimg.optimize_file()` |
| Error mapping | `SlimgError` hierarchy preserved as Python exceptions |

## CI/CD

### 1. python-bindings.yml (PR/push build & test)

```yaml
name: Python Bindings
on:
  push:
    branches: [main]
    paths:
      - 'crates/slimg-ffi/**'
      - 'crates/slimg-core/**'
      - 'bindings/python/**'
      - '.github/workflows/python-bindings.yml'
  pull_request:
    paths: # same paths

jobs:
  build-and-test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest, windows-latest]
        python-version: ['3.9', '3.13']
    steps:
      - checkout, rust toolchain, python setup
      - system dependencies (same as kotlin workflow)
      - pip install maturin pytest
      - maturin develop (working-directory: bindings/python)
      - pytest tests/ (working-directory: bindings/python)
```

### 2. publish.yml additions (PyPI deployment)

Added jobs alongside existing crates.io, Homebrew, Maven Central publishing:

```
build-python-wheels:
  - 5 platform matrix (same targets as Kotlin)
  - Uses PyO3/maturin-action@v1
  - Outputs: platform-specific .whl files

build-python-sdist:
  - Source distribution (.tar.gz)

publish-python:
  - needs: [build-python-wheels, build-python-sdist]
  - Uses PyPI Trusted Publisher (OIDC, no API token needed)
  - pypa/gh-action-pypi-publish@release/v1
```

### PyPI Setup Requirements

| Item | Detail |
|------|--------|
| PyPI account | Claim `slimg` package name on pypi.org |
| Trusted Publisher | PyPI -> Publishing -> `clroot/slimg` repo, `publish.yml` workflow |
| GitHub environment | Repo Settings -> Environments -> create `pypi` |
| Test deployment | First release on test.pypi.org, then production PyPI |

## Testing

pytest-based tests covering the same scope as Kotlin tests (`SlimgTest.kt`):

- **TestFormat**: extension, can_encode, from_path, from_bytes
- **TestDecode**: open file, decode bytes, dimensions/RGBA validation
- **TestConvert**: format conversion, pipeline with resize
- **TestCrop**: region crop, aspect ratio crop
- **TestExtend**: aspect ratio extend, size extend, fill colors
- **TestResize**: width, height, exact, fit, scale modes
- **TestOptimize**: bytes optimization, file optimization
- **TestError**: invalid data, unsupported encoding (JXL)
