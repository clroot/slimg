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
    return slimg.Image(width=width, height=height, data=bytes(data))


def pixel_at(image: slimg.Image, col: int, row: int) -> tuple:
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
