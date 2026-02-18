"""slimg - Fast image optimization library powered by Rust."""

from slimg._types import (
    Format,
    Image,
    Result,
    Resize,
    Crop,
    Extend,
    SlimgError,
    open,
    decode,
    convert,
    crop_image as crop,
    extend_image as extend,
    resize_image as resize,
    optimize,
    optimize_file,
)

__all__ = [
    "Format",
    "Image",
    "Result",
    "Resize",
    "Crop",
    "Extend",
    "SlimgError",
    "open",
    "decode",
    "convert",
    "crop",
    "extend",
    "resize",
    "optimize",
    "optimize_file",
]
