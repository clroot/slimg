import pytest
import slimg
from conftest import create_test_image


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


class TestOpen:
    def test_open_png_file(self, sample_image, tmp_path):
        # Write a PNG to disk, then open it
        result = slimg.convert(sample_image, format="png", quality=100)
        path = str(tmp_path / "test.png")
        result.save(path)
        image = slimg.open(path)
        assert image.width == 10
        assert image.height == 8
        assert image.format == slimg.Format.PNG

    def test_open_nonexistent_raises(self):
        with pytest.raises(slimg.SlimgError):
            slimg.open("/nonexistent/path/image.png")


class TestImage:
    def test_image_properties(self, sample_image):
        assert sample_image.width == 10
        assert sample_image.height == 8
        assert isinstance(sample_image.data, bytes)

    def test_image_data_length(self, sample_image):
        assert len(sample_image.data) == 10 * 8 * 4

    def test_image_format_default_none(self):
        img = create_test_image(2, 2)
        assert img.format is None

    def test_from_raw(self):
        data = bytes([0] * 4 * 3 * 2)
        img = slimg.Image._from_raw(3, 2, data)
        assert img.width == 3
        assert img.height == 2
        assert img.format is None
