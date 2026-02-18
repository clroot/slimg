use crate::codec::ImageData;
use crate::error::{Error, Result};

/// How to crop an image.
#[derive(Debug, Clone, PartialEq)]
pub enum CropMode {
    /// Extract a specific region: x, y offset with width x height.
    Region {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// Crop to an aspect ratio (centered). Width and height define the ratio (e.g. 16:9).
    AspectRatio { width: u32, height: u32 },
}

/// Calculate the crop region (x, y, width, height) for a given image size and crop mode.
pub fn calculate_crop_region(
    img_w: u32,
    img_h: u32,
    mode: &CropMode,
) -> Result<(u32, u32, u32, u32)> {
    match *mode {
        CropMode::Region {
            x,
            y,
            width,
            height,
        } => {
            if width == 0 || height == 0 {
                return Err(Error::Crop("crop dimensions must be non-zero".to_string()));
            }
            if x + width > img_w || y + height > img_h {
                return Err(Error::Crop(format!(
                    "crop region ({x},{y},{width},{height}) exceeds image bounds ({img_w}x{img_h})"
                )));
            }
            Ok((x, y, width, height))
        }
        CropMode::AspectRatio {
            width: rw,
            height: rh,
        } => {
            if rw == 0 || rh == 0 {
                return Err(Error::Crop("aspect ratio must be non-zero".to_string()));
            }
            let target_ratio = rw as f64 / rh as f64;
            let img_ratio = img_w as f64 / img_h as f64;

            let (crop_w, crop_h) = if img_ratio > target_ratio {
                let h = img_h;
                let w = (h as f64 * target_ratio).round() as u32;
                (w, h)
            } else {
                let w = img_w;
                let h = (w as f64 / target_ratio).round() as u32;
                (w, h)
            };

            let x = (img_w - crop_w) / 2;
            let y = (img_h - crop_h) / 2;

            Ok((x, y, crop_w, crop_h))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_valid_crop() {
        let (x, y, w, h) = calculate_crop_region(
            200,
            100,
            &CropMode::Region {
                x: 10,
                y: 20,
                width: 50,
                height: 30,
            },
        )
        .unwrap();
        assert_eq!((x, y, w, h), (10, 20, 50, 30));
    }

    #[test]
    fn region_full_image() {
        let (x, y, w, h) = calculate_crop_region(
            200,
            100,
            &CropMode::Region {
                x: 0,
                y: 0,
                width: 200,
                height: 100,
            },
        )
        .unwrap();
        assert_eq!((x, y, w, h), (0, 0, 200, 100));
    }

    #[test]
    fn region_exceeds_bounds() {
        let result = calculate_crop_region(
            200,
            100,
            &CropMode::Region {
                x: 150,
                y: 0,
                width: 100,
                height: 50,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn region_zero_width() {
        let result = calculate_crop_region(
            200,
            100,
            &CropMode::Region {
                x: 0,
                y: 0,
                width: 0,
                height: 50,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn region_zero_height() {
        let result = calculate_crop_region(
            200,
            100,
            &CropMode::Region {
                x: 0,
                y: 0,
                width: 50,
                height: 0,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn aspect_square_on_landscape() {
        let (x, y, w, h) = calculate_crop_region(
            200,
            100,
            &CropMode::AspectRatio {
                width: 1,
                height: 1,
            },
        )
        .unwrap();
        assert_eq!((x, y, w, h), (50, 0, 100, 100));
    }

    #[test]
    fn aspect_square_on_portrait() {
        let (x, y, w, h) = calculate_crop_region(
            100,
            200,
            &CropMode::AspectRatio {
                width: 1,
                height: 1,
            },
        )
        .unwrap();
        assert_eq!((x, y, w, h), (0, 50, 100, 100));
    }

    #[test]
    fn aspect_16_9_on_square() {
        let (x, y, w, h) = calculate_crop_region(
            100,
            100,
            &CropMode::AspectRatio {
                width: 16,
                height: 9,
            },
        )
        .unwrap();
        assert_eq!(w, 100);
        assert_eq!(h, 56);
        assert_eq!(x, 0);
        assert_eq!(y, 22);
    }

    #[test]
    fn aspect_same_as_image() {
        let (x, y, w, h) = calculate_crop_region(
            200,
            100,
            &CropMode::AspectRatio {
                width: 2,
                height: 1,
            },
        )
        .unwrap();
        assert_eq!((x, y, w, h), (0, 0, 200, 100));
    }

    #[test]
    fn aspect_zero_ratio() {
        let result = calculate_crop_region(
            200,
            100,
            &CropMode::AspectRatio {
                width: 0,
                height: 1,
            },
        );
        assert!(result.is_err());
    }
}
