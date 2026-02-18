use crate::codec::ImageData;
use crate::error::{Error, Result};

/// Fill color for the extended canvas region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FillColor {
    /// A solid RGBA color.
    Solid([u8; 4]),
    /// Fully transparent (RGBA 0,0,0,0).
    Transparent,
}

impl FillColor {
    /// Return the fill as an RGBA quadruplet.
    pub fn as_rgba(&self) -> [u8; 4] {
        match *self {
            FillColor::Solid(c) => c,
            FillColor::Transparent => [0, 0, 0, 0],
        }
    }
}

/// How to extend (add padding to) an image.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtendMode {
    /// Extend the canvas so the image fits the given aspect ratio (centered).
    /// `width` and `height` define the ratio (e.g. 16:9).
    AspectRatio { width: u32, height: u32 },
    /// Extend the canvas to an exact pixel size (centered).
    Size { width: u32, height: u32 },
}

/// Calculate the extended canvas dimensions and the offset at which the
/// original image should be placed.
///
/// Returns `(canvas_w, canvas_h, offset_x, offset_y)`.
pub fn calculate_extend_region(
    img_w: u32,
    img_h: u32,
    mode: &ExtendMode,
) -> Result<(u32, u32, u32, u32)> {
    match *mode {
        ExtendMode::AspectRatio {
            width: rw,
            height: rh,
        } => {
            if rw == 0 || rh == 0 {
                return Err(Error::Extend(
                    "aspect ratio must be non-zero".to_string(),
                ));
            }

            let target_ratio = rw as f64 / rh as f64;
            let img_ratio = img_w as f64 / img_h as f64;

            let (canvas_w, canvas_h) = if img_ratio < target_ratio {
                // Image is narrower than target → extend width.
                let h = img_h;
                let w = (h as f64 * target_ratio).round() as u32;
                (w, h)
            } else {
                // Image is wider than (or equal to) target → extend height.
                let w = img_w;
                let h = (w as f64 / target_ratio).round() as u32;
                (w, h)
            };

            let off_x = (canvas_w - img_w) / 2;
            let off_y = (canvas_h - img_h) / 2;

            Ok((canvas_w, canvas_h, off_x, off_y))
        }
        ExtendMode::Size { width, height } => {
            if width == 0 || height == 0 {
                return Err(Error::Extend(
                    "extend dimensions must be non-zero".to_string(),
                ));
            }
            if width < img_w || height < img_h {
                return Err(Error::Extend(format!(
                    "target size ({width}x{height}) is smaller than image ({img_w}x{img_h})"
                )));
            }

            let off_x = (width - img_w) / 2;
            let off_y = (height - img_h) / 2;

            Ok((width, height, off_x, off_y))
        }
    }
}

/// Extend an image by adding padding around it.
pub fn extend(image: &ImageData, mode: &ExtendMode, fill: &FillColor) -> Result<ImageData> {
    let (canvas_w, canvas_h, off_x, off_y) =
        calculate_extend_region(image.width, image.height, mode)?;

    // No-op: canvas matches image
    if canvas_w == image.width && canvas_h == image.height {
        return Ok(image.clone());
    }

    let bytes_per_pixel = 4usize;
    let canvas_stride = canvas_w as usize * bytes_per_pixel;
    let src_stride = image.width as usize * bytes_per_pixel;

    // Fill canvas with background color
    let fill_rgba = fill.as_rgba();
    let mut data = vec![0u8; canvas_h as usize * canvas_stride];
    for pixel in data.chunks_exact_mut(bytes_per_pixel) {
        pixel.copy_from_slice(&fill_rgba);
    }

    // Copy original image rows into canvas at offset
    for row in 0..image.height as usize {
        let src_offset = row * src_stride;
        let dst_offset = (off_y as usize + row) * canvas_stride + off_x as usize * bytes_per_pixel;
        data[dst_offset..dst_offset + src_stride]
            .copy_from_slice(&image.data[src_offset..src_offset + src_stride]);
    }

    Ok(ImageData::new(canvas_w, canvas_h, data))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AspectRatio tests ───────────────────────────────────────────

    #[test]
    fn aspect_square_on_landscape() {
        let (w, h, ox, oy) = calculate_extend_region(
            200,
            100,
            &ExtendMode::AspectRatio {
                width: 1,
                height: 1,
            },
        )
        .unwrap();
        assert_eq!((w, h, ox, oy), (200, 200, 0, 50));
    }

    #[test]
    fn aspect_square_on_portrait() {
        let (w, h, ox, oy) = calculate_extend_region(
            100,
            200,
            &ExtendMode::AspectRatio {
                width: 1,
                height: 1,
            },
        )
        .unwrap();
        assert_eq!((w, h, ox, oy), (200, 200, 50, 0));
    }

    #[test]
    fn aspect_16_9_on_square() {
        let (w, h, ox, oy) = calculate_extend_region(
            100,
            100,
            &ExtendMode::AspectRatio {
                width: 16,
                height: 9,
            },
        )
        .unwrap();
        assert_eq!((w, h, ox, oy), (178, 100, 39, 0));
    }

    #[test]
    fn aspect_9_16_on_square() {
        let (w, h, ox, oy) = calculate_extend_region(
            100,
            100,
            &ExtendMode::AspectRatio {
                width: 9,
                height: 16,
            },
        )
        .unwrap();
        assert_eq!((w, h, ox, oy), (100, 178, 0, 39));
    }

    #[test]
    fn aspect_same_as_image() {
        let (w, h, ox, oy) = calculate_extend_region(
            200,
            100,
            &ExtendMode::AspectRatio {
                width: 2,
                height: 1,
            },
        )
        .unwrap();
        assert_eq!((w, h, ox, oy), (200, 100, 0, 0));
    }

    #[test]
    fn aspect_zero_ratio_errors() {
        let result = calculate_extend_region(
            200,
            100,
            &ExtendMode::AspectRatio {
                width: 0,
                height: 1,
            },
        );
        assert!(result.is_err());
    }

    // ── Size tests ──────────────────────────────────────────────────

    #[test]
    fn size_larger_canvas() {
        let (w, h, ox, oy) = calculate_extend_region(
            800,
            600,
            &ExtendMode::Size {
                width: 1000,
                height: 1000,
            },
        )
        .unwrap();
        assert_eq!((w, h, ox, oy), (1000, 1000, 100, 200));
    }

    #[test]
    fn size_same_as_image() {
        let (w, h, ox, oy) = calculate_extend_region(
            800,
            600,
            &ExtendMode::Size {
                width: 800,
                height: 600,
            },
        )
        .unwrap();
        assert_eq!((w, h, ox, oy), (800, 600, 0, 0));
    }

    #[test]
    fn size_only_width_larger() {
        let (w, h, ox, oy) = calculate_extend_region(
            800,
            600,
            &ExtendMode::Size {
                width: 1000,
                height: 600,
            },
        )
        .unwrap();
        assert_eq!((w, h, ox, oy), (1000, 600, 100, 0));
    }

    #[test]
    fn size_smaller_than_image_errors() {
        let result = calculate_extend_region(
            800,
            600,
            &ExtendMode::Size {
                width: 500,
                height: 500,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn size_width_smaller_errors() {
        let result = calculate_extend_region(
            800,
            600,
            &ExtendMode::Size {
                width: 700,
                height: 600,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn size_zero_errors() {
        let result = calculate_extend_region(
            800,
            600,
            &ExtendMode::Size {
                width: 0,
                height: 0,
            },
        );
        assert!(result.is_err());
    }

    // ── extend() pixel tests ──────────────────────────────────────

    use crate::codec::ImageData;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let data = vec![128u8; (width * height * 4) as usize];
        ImageData::new(width, height, data)
    }

    #[test]
    fn extend_returns_correct_dimensions() {
        let img = create_test_image(200, 100);
        let result = extend(
            &img,
            &ExtendMode::AspectRatio {
                width: 1,
                height: 1,
            },
            &FillColor::Solid([255, 255, 255, 255]),
        )
        .unwrap();
        assert_eq!(result.width, 200);
        assert_eq!(result.height, 200);
        assert_eq!(result.data.len(), (200 * 200 * 4) as usize);
    }

    #[test]
    fn extend_fills_with_solid_color() {
        let img = create_test_image(2, 2);
        let result = extend(
            &img,
            &ExtendMode::Size {
                width: 4,
                height: 4,
            },
            &FillColor::Solid([255, 0, 0, 255]),
        )
        .unwrap();
        // Top-left corner (0,0) should be fill color (red)
        assert_eq!(result.data[0], 255); // R
        assert_eq!(result.data[1], 0); // G
        assert_eq!(result.data[2], 0); // B
        assert_eq!(result.data[3], 255); // A
    }

    #[test]
    fn extend_fills_with_transparent() {
        let img = create_test_image(2, 2);
        let result = extend(
            &img,
            &ExtendMode::Size {
                width: 4,
                height: 4,
            },
            &FillColor::Transparent,
        )
        .unwrap();
        assert_eq!(&result.data[0..4], &[0, 0, 0, 0]);
    }

    #[test]
    fn extend_preserves_pixel_data() {
        // Create 2x1 image: pixel(0,0)=[10,20,30,255] pixel(1,0)=[40,50,60,255]
        let data = vec![10, 20, 30, 255, 40, 50, 60, 255];
        let img = ImageData::new(2, 1, data);

        // Extend to 4x3 → original centered at offset (1, 1)
        let result = extend(
            &img,
            &ExtendMode::Size {
                width: 4,
                height: 3,
            },
            &FillColor::Solid([0, 0, 0, 0]),
        )
        .unwrap();

        let stride = 4 * 4; // 4 pixels * 4 bytes
        let offset = 1 * stride + 1 * 4; // row 1, col 1
        assert_eq!(&result.data[offset..offset + 4], &[10, 20, 30, 255]);

        let offset2 = 1 * stride + 2 * 4;
        assert_eq!(&result.data[offset2..offset2 + 4], &[40, 50, 60, 255]);
    }

    #[test]
    fn extend_noop_when_already_matching() {
        let img = create_test_image(200, 100);
        let result = extend(
            &img,
            &ExtendMode::AspectRatio {
                width: 2,
                height: 1,
            },
            &FillColor::Solid([255, 255, 255, 255]),
        )
        .unwrap();
        assert_eq!(result.width, 200);
        assert_eq!(result.height, 100);
        assert_eq!(result.data, img.data);
    }
}
