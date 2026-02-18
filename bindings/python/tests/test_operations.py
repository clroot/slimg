import pytest
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
        assert pixel[0] == 1  # R = row 1
        assert pixel[1] == 2  # G = col 2

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

    def test_both_args_raises(self, sample_image):
        with pytest.raises(ValueError, match="either region or aspect_ratio"):
            slimg.crop(sample_image, region=(0, 0, 5, 4), aspect_ratio=(1, 1))

    def test_no_args_raises(self, sample_image):
        with pytest.raises(ValueError, match="region or aspect_ratio"):
            slimg.crop(sample_image)


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

    def test_rgba_fill(self):
        img = create_test_image(4, 4)
        extended = slimg.extend(img, size=(6, 6), fill=(128, 64, 32, 200))
        pixel = pixel_at(extended, 0, 0)
        assert pixel == (128, 64, 32, 200)

    def test_both_args_raises(self, sample_image):
        with pytest.raises(ValueError, match="either aspect_ratio or size"):
            slimg.extend(
                sample_image,
                aspect_ratio=(1, 1),
                size=(20, 20),
                fill="transparent",
            )

    def test_no_args_raises(self, sample_image):
        with pytest.raises(ValueError, match="aspect_ratio or size"):
            slimg.extend(sample_image, fill="transparent")

    def test_fill_channel_out_of_range(self, sample_image):
        with pytest.raises(ValueError, match="0-255"):
            slimg.extend(sample_image, size=(20, 20), fill=(256, 0, 0))

    def test_fill_channel_negative(self, sample_image):
        with pytest.raises(ValueError, match="0-255"):
            slimg.extend(sample_image, size=(20, 20), fill=(-1, 0, 0))

    def test_fill_invalid_type(self, sample_image):
        with pytest.raises(ValueError, match="Invalid fill"):
            slimg.extend(sample_image, size=(20, 20), fill="red")

    def test_fill_wrong_tuple_length(self, sample_image):
        with pytest.raises(ValueError, match="Invalid fill"):
            slimg.extend(sample_image, size=(20, 20), fill=(255, 0))


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

    def test_no_mode_raises(self, sample_image):
        with pytest.raises(ValueError, match="exactly one"):
            slimg.resize(sample_image)

    def test_multiple_modes_raises(self, sample_image):
        with pytest.raises(ValueError, match="exactly one"):
            slimg.resize(sample_image, width=5, height=4)
