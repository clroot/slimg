# Python Bindings Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** slimg에 Python 바인딩을 추가하고, PyPI 배포를 기존 CI/CD 워크플로우에 통합한다.

**Architecture:** UniFFI 자동생성 Python 코드를 `_lowlevel` 내부 모듈로 격리하고, `__init__.py`에서 Pythonic 래퍼를 제공. maturin으로 빌드하여 플랫폼별 wheel을 생성.

**Tech Stack:** Rust (slimg-ffi), UniFFI 0.31, maturin, pytest, GitHub Actions, PyPI Trusted Publisher

**Design doc:** `docs/plans/2026-02-19-python-bindings-design.md`

---

### Task 1: Project scaffold & maturin build verification

**Files:**
- Create: `bindings/python/pyproject.toml`
- Create: `bindings/python/slimg/__init__.py` (minimal stub)
- Create: `bindings/python/slimg/py.typed`
- Modify: `crates/slimg-ffi/uniffi.toml`

**Step 1: Update uniffi.toml to add Python config**

```toml
# crates/slimg-ffi/uniffi.toml
[bindings.kotlin]
package_name = "io.clroot.slimg"
cdylib_name = "slimg_ffi"
generate_immutable_records = true

[bindings.python]
cdylib_name = "slimg_ffi"
```

**Step 2: Create pyproject.toml**

```toml
# bindings/python/pyproject.toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "slimg"
version = "0.3.0"
requires-python = ">=3.9"
description = "Fast image optimization library powered by Rust"
license = { text = "MIT OR Apache-2.0" }
keywords = ["image", "optimization", "compression", "webp", "avif", "jpeg", "png"]
classifiers = [
    "Development Status :: 4 - Beta",
    "License :: OSI Approved :: MIT License",
    "License :: OSI Approved :: Apache Software License",
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Python :: 3.13",
    "Topic :: Multimedia :: Graphics",
    "Topic :: Multimedia :: Graphics :: Graphics Conversion",
]

[project.urls]
Homepage = "https://github.com/clroot/slimg"
Repository = "https://github.com/clroot/slimg"
Issues = "https://github.com/clroot/slimg/issues"

[tool.maturin]
manifest-path = "../../crates/slimg-ffi/Cargo.toml"
bindings = "uniffi"
python-source = "."
module-name = "slimg._lowlevel"
```

**Step 3: Create minimal slimg/__init__.py stub**

```python
# bindings/python/slimg/__init__.py
"""slimg - Fast image optimization library powered by Rust."""

from slimg._lowlevel import *  # noqa: F401, F403
```

**Step 4: Create py.typed marker**

```
# bindings/python/slimg/py.typed
# (empty file — PEP 561 marker)
```

**Step 5: Verify maturin develop works**

```bash
cd bindings/python
pip install maturin
maturin develop
```

Expected: Build succeeds, `import slimg` works in Python.

**Step 6: Verify UniFFI-generated functions are accessible**

```bash
python -c "from slimg._lowlevel import format_extension; print(format_extension('Jpeg'))"
```

Expected: Prints `jpg` (or similar — note the exact enum variant name will depend on UniFFI's Python codegen, adjust accordingly).

**Step 7: Inspect the generated Python module to understand UniFFI's naming conventions**

```bash
python -c "import slimg._lowlevel; print(dir(slimg._lowlevel))"
```

Record the output — this determines the exact import names and enum variant conventions for the Pythonic wrapper. The wrapper implementation in subsequent tasks will reference these names.

**Step 8: Commit**

```bash
git add bindings/python/pyproject.toml bindings/python/slimg/__init__.py bindings/python/slimg/py.typed crates/slimg-ffi/uniffi.toml
git commit -m "feat(python): scaffold Python bindings with maturin + uniffi"
```

---

### Task 2: Pythonic wrapper — Format enum & utilities

**Files:**
- Create: `bindings/python/slimg/_types.py`
- Modify: `bindings/python/slimg/__init__.py`
- Create: `bindings/python/tests/conftest.py`
- Create: `bindings/python/tests/test_format.py`

**Step 1: Write failing tests for Format**

```python
# bindings/python/tests/test_format.py
import slimg


class TestFormatExtension:
    def test_jpeg(self):
        assert slimg.Format.JPEG.extension == "jpg"

    def test_png(self):
        assert slimg.Format.PNG.extension == "png"

    def test_webp(self):
        assert slimg.Format.WEBP.extension == "webp"

    def test_avif(self):
        assert slimg.Format.AVIF.extension == "avif"

    def test_jxl(self):
        assert slimg.Format.JXL.extension == "jxl"

    def test_qoi(self):
        assert slimg.Format.QOI.extension == "qoi"


class TestFormatCanEncode:
    def test_jpeg_can_encode(self):
        assert slimg.Format.JPEG.can_encode is True

    def test_jxl_cannot_encode(self):
        assert slimg.Format.JXL.can_encode is False


class TestFormatFromPath:
    def test_jpg(self):
        assert slimg.Format.from_path("photo.jpg") == slimg.Format.JPEG

    def test_jpeg(self):
        assert slimg.Format.from_path("photo.jpeg") == slimg.Format.JPEG

    def test_webp(self):
        assert slimg.Format.from_path("image.webp") == slimg.Format.WEBP

    def test_unknown_returns_none(self):
        assert slimg.Format.from_path("file.bmp") is None

    def test_no_extension_returns_none(self):
        assert slimg.Format.from_path("noext") is None


class TestFormatFromBytes:
    def test_jpeg_magic(self):
        header = bytes([0xFF, 0xD8, 0xFF, 0xE0])
        assert slimg.Format.from_bytes(header) == slimg.Format.JPEG

    def test_png_magic(self):
        header = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
        assert slimg.Format.from_bytes(header) == slimg.Format.PNG

    def test_unknown_returns_none(self):
        assert slimg.Format.from_bytes(bytes([0x00, 0x00, 0x00, 0x00])) is None
```

**Step 2: Run tests to verify they fail**

```bash
cd bindings/python
pytest tests/test_format.py -v
```

Expected: FAIL — `slimg.Format` does not have `extension`, `can_encode`, `from_path`, `from_bytes`.

**Step 3: Implement Format wrapper and _types.py**

Implement `bindings/python/slimg/_types.py` with the `Format` enum that wraps UniFFI-generated types. Use the naming conventions discovered in Task 1, Step 7.

The wrapper `Format` enum should:
- Expose `JPEG`, `PNG`, `WEBP`, `AVIF`, `JXL`, `QOI` as class-level attributes
- Provide `extension` property (calls `_lowlevel.format_extension`)
- Provide `can_encode` property (calls `_lowlevel.format_can_encode`)
- Provide `from_path(path)` classmethod (calls `_lowlevel.format_from_extension`)
- Provide `from_bytes(data)` classmethod (calls `_lowlevel.format_from_magic_bytes`)
- Support equality comparison with other Format instances

Update `__init__.py` to import and expose `Format` from `_types`.

**Step 4: Run tests to verify they pass**

```bash
cd bindings/python
pytest tests/test_format.py -v
```

Expected: All PASS.

**Step 5: Commit**

```bash
git add bindings/python/slimg/_types.py bindings/python/slimg/__init__.py bindings/python/tests/
git commit -m "feat(python): add Format enum with Pythonic wrapper"
```

---

### Task 3: Pythonic wrapper — Image (decode/open) & conftest

**Files:**
- Modify: `bindings/python/slimg/__init__.py`
- Modify: `bindings/python/slimg/_types.py`
- Create: `bindings/python/tests/conftest.py`
- Create: `bindings/python/tests/test_decode.py`

**Step 1: Create conftest.py with test helpers**

```python
# bindings/python/tests/conftest.py
import pytest
import slimg


def create_test_image(width: int, height: int) -> slimg.Image:
    """Create a test RGBA image. R=row, G=col, B=0xFF, A=0xFF."""
    data = bytearray(width * height * 4)
    for row in range(height):
        for col in range(width):
            offset = (row * width + col) * 4
            data[offset] = row & 0xFF
            data[offset + 1] = col & 0xFF
            data[offset + 2] = 0xFF
            data[offset + 3] = 0xFF
    return slimg.Image._from_raw(width, height, bytes(data))


def pixel_at(image: slimg.Image, col: int, row: int) -> tuple[int, int, int, int]:
    """Get RGBA pixel value at (col, row)."""
    offset = (row * image.width + col) * 4
    d = image.data
    return (d[offset], d[offset + 1], d[offset + 2], d[offset + 3])


@pytest.fixture
def sample_image():
    """10x8 test image."""
    return create_test_image(10, 8)


@pytest.fixture
def sample_image_100():
    """100x100 test image."""
    return create_test_image(100, 100)
```

**Step 2: Write failing tests for decode**

```python
# bindings/python/tests/test_decode.py
import pytest
import slimg
from conftest import create_test_image, pixel_at


class TestDecode:
    def test_decode_png_bytes(self, sample_image):
        # Encode to PNG first, then decode
        result = slimg.convert(sample_image, format="png", quality=100)
        image = slimg.decode(result.data)
        assert image.width == 10
        assert image.height == 8
        assert image.format == slimg.Format.PNG

    def test_decode_image_data_is_rgba(self, sample_image):
        result = slimg.convert(sample_image, format="png", quality=100)
        image = slimg.decode(result.data)
        assert len(image.data) == image.width * image.height * 4

    def test_decode_invalid_data_raises(self):
        with pytest.raises(slimg.SlimgError):
            slimg.decode(b"\x00\x00\x00\x00")


class TestImage:
    def test_image_properties(self, sample_image):
        assert sample_image.width == 10
        assert sample_image.height == 8
        assert isinstance(sample_image.data, bytes)
```

**Step 3: Run tests to verify they fail**

```bash
cd bindings/python
pytest tests/test_decode.py -v
```

Expected: FAIL — `slimg.Image`, `slimg.decode`, `slimg.convert` not yet implemented.

**Step 4: Implement Image class and decode/open functions**

Add to `_types.py`:
- `Image` class wrapping UniFFI's `ImageData`, with `width`, `height`, `data`, `format` properties
- `Image._from_raw(width, height, data)` internal constructor (for test fixtures)
- `decode(data: bytes) -> Image` function
- `open(path: str) -> Image` function (calls `_lowlevel.decode_file`)
- `SlimgError` exception re-export from `_lowlevel`

Update `__init__.py` to expose `Image`, `decode`, `open`, `SlimgError`.

**Step 5: Run tests to verify they pass**

```bash
cd bindings/python
pytest tests/test_decode.py -v
```

Expected: All PASS.

**Step 6: Commit**

```bash
git add bindings/python/
git commit -m "feat(python): add Image class, decode, and open functions"
```

---

### Task 4: Pythonic wrapper — convert & Result.save()

**Files:**
- Modify: `bindings/python/slimg/_types.py`
- Modify: `bindings/python/slimg/__init__.py`
- Create: `bindings/python/tests/test_convert.py`

**Step 1: Write failing tests**

```python
# bindings/python/tests/test_convert.py
import os
import tempfile
import slimg
from conftest import create_test_image


class TestConvert:
    def test_to_png(self, sample_image):
        result = slimg.convert(sample_image, format="png", quality=80)
        assert result.format == slimg.Format.PNG
        assert len(result.data) > 0
        # PNG magic bytes
        assert result.data[0] == 0x89
        assert result.data[1] == 0x50

    def test_to_webp(self, sample_image):
        result = slimg.convert(sample_image, format="webp", quality=75)
        assert result.format == slimg.Format.WEBP
        # RIFF magic
        assert result.data[:4] == b"RIFF"

    def test_with_resize(self, sample_image):
        result = slimg.convert(
            sample_image, format="png", quality=80,
            resize=slimg.Resize.width(5),
        )
        decoded = slimg.decode(result.data)
        assert decoded.width == 5
        assert decoded.height == 4  # aspect ratio preserved (10x8 -> 5x4)

    def test_format_string_case_insensitive(self, sample_image):
        result = slimg.convert(sample_image, format="PNG", quality=80)
        assert result.format == slimg.Format.PNG

    def test_format_enum_accepted(self, sample_image):
        result = slimg.convert(sample_image, format=slimg.Format.PNG, quality=80)
        assert result.format == slimg.Format.PNG

    def test_jxl_encode_raises(self, sample_image):
        with pytest.raises(slimg.SlimgError):
            slimg.convert(sample_image, format="jxl", quality=80)

    def test_full_pipeline_crop_extend(self, sample_image_100):
        result = slimg.convert(
            sample_image_100,
            format="png",
            quality=80,
            crop=slimg.Crop.aspect_ratio(16, 9),
            extend=slimg.Extend.aspect_ratio(1, 1),
            fill=(255, 255, 255),
        )
        decoded = slimg.decode(result.data)
        assert decoded.width == decoded.height  # square after extend


class TestResultSave:
    def test_save_to_file(self, sample_image):
        result = slimg.convert(sample_image, format="png", quality=80)
        with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
            path = f.name
        try:
            result.save(path)
            assert os.path.getsize(path) == len(result.data)
        finally:
            os.unlink(path)
```

**Step 2: Run tests to verify they fail**

```bash
cd bindings/python
pytest tests/test_convert.py -v
```

Expected: FAIL — `slimg.convert`, `slimg.Resize`, `slimg.Crop`, `slimg.Extend`, `Result.save` not implemented.

**Step 3: Implement convert, Resize, Crop, Extend helpers, and Result class**

Add to `_types.py`:
- `Result` class with `data`, `format` properties and `save(path)` method
- `Resize` namespace class with static methods: `width(v)`, `height(v)`, `exact(w, h)`, `fit(w, h)`, `scale(f)`
- `Crop` namespace class: `region(x, y, w, h)`, `aspect_ratio(w, h)`
- `Extend` namespace class: `aspect_ratio(w, h)`, `size(w, h)`
- `convert(image, format, quality, *, resize=None, crop=None, extend=None, fill=None)` function
  - `format` accepts string or `Format` enum
  - Converts string to UniFFI enum internally
  - `fill` accepts `"transparent"` or `(r, g, b)` or `(r, g, b, a)` tuple

Update `__init__.py` to expose all new types.

**Step 4: Run tests to verify they pass**

```bash
cd bindings/python
pytest tests/test_convert.py -v
```

Expected: All PASS.

**Step 5: Commit**

```bash
git add bindings/python/
git commit -m "feat(python): add convert pipeline with Resize, Crop, Extend helpers"
```

---

### Task 5: Pythonic wrapper — crop, extend, resize standalone functions

**Files:**
- Modify: `bindings/python/slimg/_types.py`
- Modify: `bindings/python/slimg/__init__.py`
- Create: `bindings/python/tests/test_operations.py`

**Step 1: Write failing tests**

```python
# bindings/python/tests/test_operations.py
import slimg
from conftest import create_test_image, pixel_at


class TestCrop:
    def test_region(self, sample_image):
        cropped = slimg.crop(sample_image, region=(2, 1, 5, 4))
        assert cropped.width == 5
        assert cropped.height == 4

    def test_region_preserves_pixels(self, sample_image):
        cropped = slimg.crop(sample_image, region=(2, 1, 3, 2))
        pixel = pixel_at(cropped, 0, 0)
        assert pixel[0] == 1   # R = row 1
        assert pixel[1] == 2   # G = col 2

    def test_aspect_ratio_square(self, sample_image):
        cropped = slimg.crop(sample_image, aspect_ratio=(1, 1))
        assert cropped.width == cropped.height

    def test_aspect_ratio_16_9(self, sample_image_100):
        cropped = slimg.crop(sample_image_100, aspect_ratio=(16, 9))
        ratio = cropped.width / cropped.height
        assert 1.7 < ratio < 1.8

    def test_out_of_bounds_raises(self, sample_image):
        with pytest.raises(slimg.SlimgError):
            slimg.crop(sample_image, region=(8, 0, 5, 4))


class TestExtend:
    def test_aspect_ratio_square_from_landscape(self, sample_image):
        extended = slimg.extend(sample_image, aspect_ratio=(1, 1), fill="transparent")
        assert extended.width == 10
        assert extended.height == 10

    def test_aspect_ratio_square_from_portrait(self):
        img = create_test_image(6, 10)
        extended = slimg.extend(img, aspect_ratio=(1, 1), fill="transparent")
        assert extended.width == 10
        assert extended.height == 10

    def test_solid_fill(self):
        img = create_test_image(4, 4)
        extended = slimg.extend(img, size=(6, 6), fill=(255, 0, 0))
        pixel = pixel_at(extended, 0, 0)
        assert pixel == (255, 0, 0, 255)

    def test_transparent_fill(self):
        img = create_test_image(4, 4)
        extended = slimg.extend(img, size=(6, 6), fill="transparent")
        pixel = pixel_at(extended, 0, 0)
        assert pixel == (0, 0, 0, 0)

    def test_preserves_original(self):
        img = create_test_image(4, 4)
        extended = slimg.extend(img, size=(6, 6), fill="transparent")
        # 4x4 centered in 6x6 -> offset (1,1)
        pixel = pixel_at(extended, 1, 1)
        assert pixel == (0, 0, 0xFF, 0xFF)

    def test_noop_when_matching(self):
        img = create_test_image(10, 10)
        extended = slimg.extend(img, aspect_ratio=(1, 1), fill="transparent")
        assert extended.data == img.data

    def test_smaller_size_raises(self, sample_image):
        with pytest.raises(slimg.SlimgError):
            slimg.extend(sample_image, size=(5, 8), fill="transparent")


class TestResize:
    def test_width(self, sample_image):
        resized = slimg.resize(sample_image, width=5)
        assert resized.width == 5
        assert resized.height == 4  # 10x8 -> 5x4

    def test_height(self, sample_image):
        resized = slimg.resize(sample_image, height=4)
        assert resized.height == 4

    def test_exact(self, sample_image):
        resized = slimg.resize(sample_image, exact=(20, 20))
        assert resized.width == 20
        assert resized.height == 20

    def test_fit(self, sample_image):
        resized = slimg.resize(sample_image, fit=(5, 5))
        assert resized.width <= 5
        assert resized.height <= 5

    def test_scale(self, sample_image):
        resized = slimg.resize(sample_image, scale=2.0)
        assert resized.width == 20
        assert resized.height == 16
```

**Step 2: Run tests to verify they fail**

```bash
cd bindings/python
pytest tests/test_operations.py -v
```

Expected: FAIL — `slimg.crop`, `slimg.extend`, `slimg.resize` not implemented as standalone functions.

**Step 3: Implement standalone crop, extend, resize functions**

Add to `_types.py`:
- `crop(image, *, region=None, aspect_ratio=None) -> Image`
  - Exactly one of `region` or `aspect_ratio` must be provided
  - `region=(x, y, w, h)` tuple -> `CropMode.Region`
  - `aspect_ratio=(w, h)` tuple -> `CropMode.AspectRatio`
- `extend(image, *, aspect_ratio=None, size=None, fill) -> Image`
  - Exactly one of `aspect_ratio` or `size` must be provided
  - `fill` accepts `"transparent"`, `(r, g, b)`, or `(r, g, b, a)`
- `resize(image, *, width=None, height=None, exact=None, fit=None, scale=None) -> Image`
  - Exactly one resize mode must be provided

Update `__init__.py` to expose `crop`, `extend`, `resize`.

**Step 4: Run tests to verify they pass**

```bash
cd bindings/python
pytest tests/test_operations.py -v
```

Expected: All PASS.

**Step 5: Commit**

```bash
git add bindings/python/
git commit -m "feat(python): add standalone crop, extend, resize functions"
```

---

### Task 6: Pythonic wrapper — optimize & optimize_file

**Files:**
- Modify: `bindings/python/slimg/_types.py`
- Modify: `bindings/python/slimg/__init__.py`
- Create: `bindings/python/tests/test_optimize.py`

**Step 1: Write failing tests**

```python
# bindings/python/tests/test_optimize.py
import os
import tempfile
import slimg
from conftest import create_test_image


class TestOptimize:
    def test_optimize_png_bytes(self, sample_image):
        encoded = slimg.convert(sample_image, format="png", quality=80)
        result = slimg.optimize(encoded.data, quality=60)
        assert result.format == slimg.Format.PNG
        assert len(result.data) > 0

    def test_optimize_preserves_format(self, sample_image):
        encoded = slimg.convert(sample_image, format="webp", quality=80)
        result = slimg.optimize(encoded.data, quality=60)
        assert result.format == slimg.Format.WEBP

    def test_optimize_invalid_data_raises(self):
        with pytest.raises(slimg.SlimgError):
            slimg.optimize(b"\x00\x00\x00\x00", quality=80)


class TestOptimizeFile:
    def test_optimize_file(self, sample_image):
        encoded = slimg.convert(sample_image, format="png", quality=100)
        with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
            f.write(encoded.data)
            path = f.name
        try:
            result = slimg.optimize_file(path, quality=60)
            assert result.format == slimg.Format.PNG
            assert len(result.data) > 0
        finally:
            os.unlink(path)
```

**Step 2: Run tests to verify they fail**

```bash
cd bindings/python
pytest tests/test_optimize.py -v
```

Expected: FAIL — `slimg.optimize`, `slimg.optimize_file` not implemented.

**Step 3: Implement optimize and optimize_file**

Add to `_types.py`:
- `optimize(data: bytes, quality: int) -> Result`
- `optimize_file(path: str, quality: int) -> Result` — reads file, calls optimize

Update `__init__.py` to expose both.

**Step 4: Run tests to verify they pass**

```bash
cd bindings/python
pytest tests/test_optimize.py -v
```

Expected: All PASS.

**Step 5: Commit**

```bash
git add bindings/python/
git commit -m "feat(python): add optimize and optimize_file functions"
```

---

### Task 7: Run full test suite & verify public API completeness

**Step 1: Run all tests together**

```bash
cd bindings/python
pytest tests/ -v
```

Expected: All PASS.

**Step 2: Verify public API surface**

```bash
python -c "
import slimg
expected = ['Format', 'Image', 'Result', 'Resize', 'Crop', 'Extend', 'SlimgError',
            'open', 'decode', 'convert', 'crop', 'extend', 'resize', 'optimize', 'optimize_file']
for name in expected:
    assert hasattr(slimg, name), f'Missing: {name}'
print('All public API members present')
"
```

Expected: `All public API members present`.

**Step 3: Verify __all__ is defined in __init__.py**

Check that `__init__.py` defines `__all__` listing the public API to prevent leaking internal names.

**Step 4: Commit if any fixes were needed**

```bash
git add bindings/python/
git commit -m "fix(python): ensure complete public API surface"
```

---

### Task 8: CI — python-bindings.yml

**Files:**
- Create: `.github/workflows/python-bindings.yml`

**Step 1: Create workflow file**

Reference the existing `kotlin-bindings.yml` for system dependency installation patterns. The Python workflow should:

- Trigger on push to main and PR, filtered by paths: `crates/slimg-ffi/**`, `crates/slimg-core/**`, `bindings/python/**`, `.github/workflows/python-bindings.yml`
- Matrix: `os: [macos-latest, ubuntu-latest, windows-latest]` × `python-version: ['3.9', '3.13']`
- Steps:
  1. `actions/checkout@v4`
  2. `dtolnay/rust-toolchain@stable`
  3. `actions/setup-python@v5` with matrix python-version
  4. System dependencies — copy exact steps from `kotlin-bindings.yml` (nasm, meson, ninja for each OS, MSVC setup for Windows, `SYSTEM_DEPS_DAV1D_BUILD_INTERNAL=always`)
  5. `pip install maturin pytest`
  6. `maturin develop` (working-directory: `bindings/python`)
  7. `pytest tests/ -v` (working-directory: `bindings/python`)

**Step 2: Commit**

```bash
git add .github/workflows/python-bindings.yml
git commit -m "ci: add Python bindings build and test workflow"
```

---

### Task 9: CI — publish.yml PyPI integration

**Files:**
- Modify: `.github/workflows/publish.yml`

**Step 1: Add build-python-wheels job**

Add after the existing `publish-kotlin` job. Use matrix matching the Kotlin native build targets:

```yaml
build-python-wheels:
  name: Build Python wheel (${{ matrix.target }})
  runs-on: ${{ matrix.runner }}
  env:
    SYSTEM_DEPS_DAV1D_BUILD_INTERNAL: always
  strategy:
    fail-fast: false
    matrix:
      include:
        - { target: aarch64-apple-darwin,        runner: macos-latest }
        - { target: x86_64-apple-darwin,         runner: macos-13 }
        - { target: x86_64-unknown-linux-gnu,    runner: ubuntu-latest }
        - { target: aarch64-unknown-linux-gnu,   runner: ubuntu-24.04-arm }
        - { target: x86_64-pc-windows-msvc,      runner: windows-latest }
  steps:
    - uses: actions/checkout@v4
    # System dependencies — same pattern as build-kotlin-native job
    # (nasm, meson, ninja, MSVC setup, dav1d flag)
    - name: Build wheel
      uses: PyO3/maturin-action@v1
      with:
        target: ${{ matrix.target }}
        args: --release --out dist
        working-directory: bindings/python
    - uses: actions/upload-artifact@v4
      with:
        name: python-wheels-${{ matrix.target }}
        path: bindings/python/dist/*.whl
```

**Step 2: Add build-python-sdist job**

```yaml
build-python-sdist:
  name: Build Python sdist
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Build sdist
      uses: PyO3/maturin-action@v1
      with:
        command: sdist
        args: --out dist
        working-directory: bindings/python
    - uses: actions/upload-artifact@v4
      with:
        name: python-wheels-sdist
        path: bindings/python/dist/*.tar.gz
```

**Step 3: Add publish-python job**

```yaml
publish-python:
  name: Publish to PyPI
  needs: [build-python-wheels, build-python-sdist]
  runs-on: ubuntu-latest
  permissions:
    id-token: write
  environment: pypi
  steps:
    - uses: actions/download-artifact@v4
      with:
        pattern: python-wheels-*
        path: dist/
        merge-multiple: true
    - name: Publish to PyPI
      uses: pypa/gh-action-pypi-publish@release/v1
      with:
        packages-dir: dist/
```

**Step 4: Commit**

```bash
git add .github/workflows/publish.yml
git commit -m "ci: add Python wheel build and PyPI publish to release workflow"
```

---

### Task 10: README for Python bindings

**Files:**
- Create: `bindings/python/README.md`

**Step 1: Write README**

Write a README covering:
- Installation: `pip install slimg`
- Quick start examples (decode, convert, crop, resize, extend, optimize)
- Supported formats table (same as main README)
- Supported platforms
- Link to main repository README

Use the same style/structure as `bindings/kotlin/README.md`.

**Step 2: Update pyproject.toml to use README**

Add to `[project]` section:
```toml
readme = "README.md"
```

**Step 3: Commit**

```bash
git add bindings/python/README.md bindings/python/pyproject.toml
git commit -m "docs: add Python bindings README"
```
