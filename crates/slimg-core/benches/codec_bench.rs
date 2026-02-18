use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use slimg_core::{
    codec::get_codec, EncodeOptions, Format, ImageData,
};

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

/// Formats that support encoding.
///
/// JXL is excluded because encoding is not supported (license restrictions).
/// AVIF encoding works on all platforms via `ravif`.
fn encodable_formats() -> Vec<Format> {
    vec![Format::Jpeg, Format::Png, Format::WebP, Format::Qoi, Format::Avif]
}

/// Formats that support both encoding and decoding (needed for decode benchmarks).
fn decodable_formats() -> Vec<Format> {
    vec![Format::Jpeg, Format::Png, Format::WebP, Format::Qoi, Format::Avif]
}

fn bench_encode(c: &mut Criterion) {
    let image = generate_test_image(BENCH_IMAGE_SIZE, BENCH_IMAGE_SIZE);
    let pixel_count = (BENCH_IMAGE_SIZE as u64) * (BENCH_IMAGE_SIZE as u64);
    let options = EncodeOptions { quality: 80 };

    let mut group = c.benchmark_group("encode");
    group.throughput(Throughput::Elements(pixel_count));

    for format in encodable_formats() {
        let codec = get_codec(format);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", format)),
            &(&image, &options),
            |b, &(image, options)| {
                b.iter(|| codec.encode(image, options).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let image = generate_test_image(BENCH_IMAGE_SIZE, BENCH_IMAGE_SIZE);
    let options = EncodeOptions { quality: 80 };

    let mut group = c.benchmark_group("decode");

    for format in decodable_formats() {
        let codec = get_codec(format);
        let encoded = codec.encode(&image, &options).unwrap();

        group.throughput(Throughput::Bytes(encoded.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", format)),
            &encoded,
            |b, data| {
                b.iter(|| codec.decode(data).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
