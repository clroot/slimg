import pytest
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

    def test_png_can_encode(self):
        assert slimg.Format.PNG.can_encode is True

    def test_webp_can_encode(self):
        assert slimg.Format.WEBP.can_encode is True

    def test_jxl_can_encode(self):
        assert slimg.Format.JXL.can_encode is True


class TestFormatFromPath:
    def test_jpg(self):
        assert slimg.Format.from_path("photo.jpg") == slimg.Format.JPEG

    def test_jpeg(self):
        assert slimg.Format.from_path("photo.jpeg") == slimg.Format.JPEG

    def test_webp(self):
        assert slimg.Format.from_path("image.webp") == slimg.Format.WEBP

    def test_png(self):
        assert slimg.Format.from_path("image.png") == slimg.Format.PNG

    def test_avif(self):
        assert slimg.Format.from_path("image.avif") == slimg.Format.AVIF

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


class TestFormatResolve:
    def test_string_lowercase(self):
        assert slimg.Format._resolve("png") == slimg.Format.PNG

    def test_string_uppercase(self):
        assert slimg.Format._resolve("PNG") == slimg.Format.PNG

    def test_string_jpg_alias(self):
        assert slimg.Format._resolve("jpg") == slimg.Format.JPEG

    def test_enum_passthrough(self):
        assert slimg.Format._resolve(slimg.Format.WEBP) == slimg.Format.WEBP

    def test_unknown_string_raises(self):
        with pytest.raises(ValueError, match="Unknown format"):
            slimg.Format._resolve("bmp")

    def test_wrong_type_raises(self):
        with pytest.raises(TypeError):
            slimg.Format._resolve(123)
