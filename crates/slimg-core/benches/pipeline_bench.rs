use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use slimg_core::codec::{get_codec, EncodeOptions};
use slimg_core::resize::resize;
use slimg_core::{convert, optimize, Format, ImageData, PipelineOptions, ResizeMode};

const BENCH_IMAGE_SIZE: u32 = 512;

fn generate_test_image(width: u32, height: u32) -> ImageData {
    let mut data = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            data[i] = (x * 255 / width) as u8;
            data[i + 1] = (y * 255 / height) as u8;
            data[i + 2] = 128;
            data[i + 3] = 255;
        }
    }
    ImageData::new(width, height, data)
}

/// Pre-encode a test image in the given format and return the encoded bytes.
fn pre_encode(image: &ImageData, format: Format, quality: u8) -> Vec<u8> {
    let codec = get_codec(format);
    let options = EncodeOptions { quality };
    codec.encode(image, &options).unwrap()
}

fn bench_convert(c: &mut Criterion) {
    let image = generate_test_image(BENCH_IMAGE_SIZE, BENCH_IMAGE_SIZE);

    let conversions: Vec<(&str, Format, Format)> = vec![
        ("jpeg_to_webp", Format::Jpeg, Format::WebP),
        ("png_to_jpeg", Format::Png, Format::Jpeg),
        ("webp_to_png", Format::WebP, Format::Png),
        ("png_to_avif", Format::Png, Format::Avif),
    ];

    let pixel_count = (BENCH_IMAGE_SIZE as u64) * (BENCH_IMAGE_SIZE as u64);
    let mut group = c.benchmark_group("convert");
    group.throughput(Throughput::Elements(pixel_count));

    for (name, src_format, dst_format) in &conversions {
        // Pre-encode the test image in the source format, then decode it.
        let encoded = pre_encode(&image, *src_format, 80);
        let codec = get_codec(*src_format);
        let decoded = codec.decode(&encoded).unwrap();

        let options = PipelineOptions {
            format: *dst_format,
            quality: 80,
            resize: None,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(&decoded, &options),
            |b, &(image, opts)| {
                b.iter(|| convert(image, opts).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_optimize(c: &mut Criterion) {
    let image = generate_test_image(BENCH_IMAGE_SIZE, BENCH_IMAGE_SIZE);

    let formats = vec![
        ("Jpeg", Format::Jpeg),
        ("Png", Format::Png),
        ("WebP", Format::WebP),
        ("Avif", Format::Avif),
    ];

    let mut group = c.benchmark_group("optimize");

    for (name, format) in &formats {
        let encoded = pre_encode(&image, *format, 90);

        group.throughput(Throughput::Bytes(encoded.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &encoded,
            |b, data| {
                b.iter(|| optimize(data, 80).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_resize(c: &mut Criterion) {
    let image = generate_test_image(BENCH_IMAGE_SIZE, BENCH_IMAGE_SIZE);

    let modes: Vec<(&str, ResizeMode)> = vec![
        ("width_256", ResizeMode::Width(256)),
        ("height_256", ResizeMode::Height(256)),
        ("exact_256x256", ResizeMode::Exact(256, 256)),
        ("scale_0.5", ResizeMode::Scale(0.5)),
        ("fit_256x256", ResizeMode::Fit(256, 256)),
    ];

    let pixel_count = (BENCH_IMAGE_SIZE as u64) * (BENCH_IMAGE_SIZE as u64);
    let mut group = c.benchmark_group("resize");
    group.throughput(Throughput::Elements(pixel_count));

    for (name, mode) in &modes {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(&image, mode),
            |b, &(image, mode)| {
                b.iter(|| resize(image, mode).unwrap());
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_convert, bench_optimize, bench_resize);
criterion_main!(benches);
