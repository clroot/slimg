use image::imageops::FilterType;
use image::{DynamicImage, RgbaImage};

use crate::codec::ImageData;
use crate::error::{Error, Result};

/// How to resize an image.
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeMode {
    /// Set width, calculate height preserving aspect ratio.
    Width(u32),
    /// Set height, calculate width preserving aspect ratio.
    Height(u32),
    /// Exact dimensions (may distort the image).
    Exact(u32, u32),
    /// Fit within bounds, preserving aspect ratio.
    Fit(u32, u32),
    /// Scale factor (e.g. 0.5 = half size).
    Scale(f64),
}

/// Calculate the target dimensions for a resize operation.
pub fn calculate_dimensions(orig_w: u32, orig_h: u32, mode: &ResizeMode) -> Result<(u32, u32)> {
    let (w, h) = match *mode {
        ResizeMode::Width(target_w) => {
            let ratio = target_w as f64 / orig_w as f64;
            let target_h = (orig_h as f64 * ratio).round() as u32;
            (target_w, target_h)
        }
        ResizeMode::Height(target_h) => {
            let ratio = target_h as f64 / orig_h as f64;
            let target_w = (orig_w as f64 * ratio).round() as u32;
            (target_w, target_h)
        }
        ResizeMode::Exact(w, h) => (w, h),
        ResizeMode::Fit(max_w, max_h) => {
            let ratio_w = max_w as f64 / orig_w as f64;
            let ratio_h = max_h as f64 / orig_h as f64;
            let ratio = ratio_w.min(ratio_h);
            let target_w = (orig_w as f64 * ratio).round() as u32;
            let target_h = (orig_h as f64 * ratio).round() as u32;
            (target_w, target_h)
        }
        ResizeMode::Scale(factor) => {
            if factor <= 0.0 {
                return Err(Error::Resize("scale factor must be positive".to_string()));
            }
            let target_w = (orig_w as f64 * factor).round() as u32;
            let target_h = (orig_h as f64 * factor).round() as u32;
            (target_w, target_h)
        }
    };

    if w == 0 || h == 0 {
        return Err(Error::Resize(format!(
            "calculated dimensions are zero: {w}x{h}"
        )));
    }

    Ok((w, h))
}

/// Resize an image according to the given mode.
pub fn resize(image: &ImageData, mode: &ResizeMode) -> Result<ImageData> {
    let (target_w, target_h) = calculate_dimensions(image.width, image.height, mode)?;

    let rgba =
        RgbaImage::from_raw(image.width, image.height, image.data.clone()).ok_or_else(|| {
            Error::Resize(format!(
                "failed to create RgbaImage from {}x{} data ({} bytes)",
                image.width,
                image.height,
                image.data.len(),
            ))
        })?;

    let dynamic = DynamicImage::ImageRgba8(rgba);

    let resized = match mode {
        ResizeMode::Exact(_, _) => dynamic.resize_exact(target_w, target_h, FilterType::Lanczos3),
        _ => dynamic.resize(target_w, target_h, FilterType::Lanczos3),
    };

    let output = resized.to_rgba8();
    Ok(ImageData::new(
        output.width(),
        output.height(),
        output.into_raw(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let data = vec![128u8; (width * height * 4) as usize];
        ImageData::new(width, height, data)
    }

    #[test]
    fn resize_by_width_preserves_ratio() {
        let img = create_test_image(200, 100);
        let result = resize(&img, &ResizeMode::Width(100)).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn resize_by_height_preserves_ratio() {
        let img = create_test_image(200, 100);
        let result = resize(&img, &ResizeMode::Height(50)).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn resize_fit_within_bounds() {
        let img = create_test_image(400, 200);
        let result = resize(&img, &ResizeMode::Fit(100, 100)).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn resize_by_scale() {
        let img = create_test_image(200, 100);
        let result = resize(&img, &ResizeMode::Scale(0.5)).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn resize_exact_ignores_ratio() {
        let img = create_test_image(200, 100);
        let result = resize(&img, &ResizeMode::Exact(50, 50)).unwrap();
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);
    }
}
