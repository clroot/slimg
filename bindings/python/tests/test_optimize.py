import os
import tempfile

import pytest
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

    def test_optimize_file_webp(self, sample_image):
        encoded = slimg.convert(sample_image, format="webp", quality=100)
        with tempfile.NamedTemporaryFile(suffix=".webp", delete=False) as f:
            f.write(encoded.data)
            path = f.name
        try:
            result = slimg.optimize_file(path, quality=60)
            assert result.format == slimg.Format.WEBP
        finally:
            os.unlink(path)

    def test_optimize_file_nonexistent_raises(self):
        with pytest.raises(FileNotFoundError):
            slimg.optimize_file("/nonexistent/path/image.png", quality=80)
