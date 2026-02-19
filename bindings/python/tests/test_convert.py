import os
import tempfile

import pytest
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

    def test_to_jpeg(self, sample_image):
        result = slimg.convert(sample_image, format="jpeg", quality=80)
        assert result.format == slimg.Format.JPEG
        # JPEG magic bytes
        assert result.data[0] == 0xFF
        assert result.data[1] == 0xD8

    def test_with_resize(self, sample_image):
        result = slimg.convert(
            sample_image,
            format="png",
            quality=80,
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

    def test_to_jxl(self, sample_image):
        result = slimg.convert(sample_image, format="jxl", quality=80)
        assert result.format == slimg.Format.JXL
        assert len(result.data) > 0

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


class TestConvertValidation:
    def test_quality_too_high(self, sample_image):
        with pytest.raises(ValueError, match="quality"):
            slimg.convert(sample_image, format="png", quality=101)

    def test_quality_negative(self, sample_image):
        with pytest.raises(ValueError, match="quality"):
            slimg.convert(sample_image, format="png", quality=-1)

    def test_quality_not_int(self, sample_image):
        with pytest.raises(ValueError, match="quality"):
            slimg.convert(sample_image, format="png", quality=80.5)

    def test_unknown_format_string(self, sample_image):
        with pytest.raises(ValueError, match="Unknown format"):
            slimg.convert(sample_image, format="bmp", quality=80)


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

    def test_save_reads_back(self, sample_image):
        result = slimg.convert(sample_image, format="png", quality=80)
        with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
            path = f.name
        try:
            result.save(path)
            with open(path, "rb") as f:
                saved = f.read()
            assert saved == result.data
        finally:
            os.unlink(path)
