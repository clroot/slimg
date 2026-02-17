use slimg_core::*;

fn create_test_image() -> ImageData {
    let (w, h) = (100, 80);
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            data[i] = (x * 255 / w) as u8;
            data[i + 1] = (y * 255 / h) as u8;
            data[i + 2] = 128;
            data[i + 3] = 255;
        }
    }
    ImageData::new(w, h, data)
}

#[test]
fn convert_jpeg_to_webp() {
    let image = create_test_image();

    // Encode as JPEG first
    let jpeg_options = PipelineOptions {
        format: Format::Jpeg,
        quality: 90,
        resize: None,
    };
    let jpeg_result = convert(&image, &jpeg_options).expect("JPEG encode failed");
    assert!(!jpeg_result.data.is_empty());
    assert_eq!(jpeg_result.format, Format::Jpeg);

    // Decode the JPEG bytes
    let (decoded, detected_format) = decode(&jpeg_result.data).expect("JPEG decode failed");
    assert_eq!(detected_format, Format::Jpeg);
    assert_eq!(decoded.width, 100);
    assert_eq!(decoded.height, 80);

    // Convert the decoded image to WebP
    let webp_options = PipelineOptions {
        format: Format::WebP,
        quality: 80,
        resize: None,
    };
    let webp_result = convert(&decoded, &webp_options).expect("WebP encode failed");
    assert!(!webp_result.data.is_empty());
    assert_eq!(webp_result.format, Format::WebP);

    // Verify the WebP bytes are detected correctly
    let detected = Format::from_magic_bytes(&webp_result.data);
    assert_eq!(detected, Some(Format::WebP));
}

#[test]
fn convert_with_resize() {
    let image = create_test_image();

    let options = PipelineOptions {
        format: Format::Png,
        quality: 80,
        resize: Some(ResizeMode::Width(50)),
    };
    let result = convert(&image, &options).expect("PNG encode with resize failed");
    assert!(!result.data.is_empty());
    assert_eq!(result.format, Format::Png);

    // Decode the resized PNG and verify dimensions
    let (decoded, format) = decode(&result.data).expect("PNG decode failed");
    assert_eq!(format, Format::Png);
    assert_eq!(decoded.width, 50);
    assert_eq!(decoded.height, 40); // 100:80 ratio preserved -> 50:40
}

#[test]
fn roundtrip_all_encodable_formats() {
    let image = create_test_image();

    let formats = [
        Format::Jpeg,
        Format::Png,
        Format::WebP,
        Format::Avif,
        Format::Qoi,
    ];

    for fmt in formats {
        let options = PipelineOptions {
            format: fmt,
            quality: 80,
            resize: None,
        };

        // Encode
        let result = convert(&image, &options).unwrap_or_else(|e| {
            panic!("encoding to {fmt:?} failed: {e}");
        });
        assert!(
            !result.data.is_empty(),
            "{fmt:?}: encoded data should not be empty",
        );

        // Decode and verify
        let (decoded, detected_format) = decode(&result.data).unwrap_or_else(|e| {
            panic!("decoding {fmt:?} failed: {e}");
        });
        assert_eq!(
            detected_format, fmt,
            "{fmt:?}: detected format mismatch",
        );
        assert_eq!(decoded.width, 100, "{fmt:?}: width mismatch");
        assert_eq!(decoded.height, 80, "{fmt:?}: height mismatch");
    }
}
