# Benchmark Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** slimg-core에 criterion 기반 벤치마크를 추가하여 코덱별 encode/decode와 파이프라인 성능을 측정한다.

**Architecture:** `crates/slimg-core/benches/` 에 두 개의 벤치마크 파일(codec_bench.rs, pipeline_bench.rs)을 추가한다. 테스트 이미지는 코드로 생성하며, criterion의 benchmark group을 활용해 포맷/모드별로 그룹화한다.

**Tech Stack:** Rust, criterion 0.5, slimg-core

---

### Task 1: Add criterion dependency and bench targets

**Files:**
- Modify: `crates/slimg-core/Cargo.toml`

**Step 1: Add criterion dev-dependency and bench targets to Cargo.toml**

`crates/slimg-core/Cargo.toml` 끝에 다음을 추가:

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "codec_bench"
harness = false

[[bench]]
name = "pipeline_bench"
harness = false
```

**Step 2: Verify the dependency resolves**

Run: `cargo check -p slimg-core`
Expected: success (bench files don't exist yet, but check only validates Cargo.toml)

**Step 3: Commit**

```bash
git add crates/slimg-core/Cargo.toml Cargo.lock
git commit -m "chore: add criterion benchmark dependency and targets"
```

---

### Task 2: Create codec_bench.rs - encode/decode benchmarks

**Files:**
- Create: `crates/slimg-core/benches/codec_bench.rs`

**Step 1: Create the codec benchmark file**

```rust
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use slimg_core::{
    Codec, EncodeOptions, Format, ImageData,
    codec::get_codec,
};

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

fn encodable_formats() -> Vec<Format> {
    let mut formats = vec![Format::Jpeg, Format::Png, Format::WebP, Format::Qoi];
    #[cfg(target_os = "macos")]
    formats.push(Format::Avif);
    formats
}

fn bench_encode(c: &mut Criterion) {
    let image = generate_test_image(512, 512);
    let pixel_count = 512u64 * 512;
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
    let image = generate_test_image(512, 512);
    let options = EncodeOptions { quality: 80 };

    let mut group = c.benchmark_group("decode");

    for format in encodable_formats() {
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
```

**Step 2: Run the codec benchmarks to verify they work**

Run: `cargo bench -p slimg-core --bench codec_bench`
Expected: Benchmark results for encode/decode per format

**Step 3: Commit**

```bash
git add crates/slimg-core/benches/codec_bench.rs
git commit -m "feat: add codec encode/decode benchmarks"
```

---

### Task 3: Create pipeline_bench.rs - pipeline benchmarks

**Files:**
- Create: `crates/slimg-core/benches/pipeline_bench.rs`

**Step 1: Create the pipeline benchmark file**

```rust
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use slimg_core::{
    EncodeOptions, Format, ImageData, PipelineOptions, ResizeMode,
    codec::get_codec,
    convert, optimize, resize::resize,
};

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

fn bench_convert(c: &mut Criterion) {
    let image = generate_test_image(512, 512);

    let conversions: Vec<(&str, Format, Format)> = {
        let mut v = vec![
            ("jpeg_to_webp", Format::Jpeg, Format::WebP),
            ("png_to_jpeg", Format::Png, Format::Jpeg),
            ("webp_to_png", Format::WebP, Format::Png),
        ];
        #[cfg(target_os = "macos")]
        v.push(("png_to_avif", Format::Png, Format::Avif));
        v
    };

    // Pre-encode source images
    let options = EncodeOptions { quality: 80 };
    let sources: Vec<(&str, ImageData, Format)> = conversions
        .iter()
        .map(|(name, src_fmt, _)| {
            let codec = get_codec(*src_fmt);
            let encoded = codec.encode(&image, &options).unwrap();
            let (decoded, _) = slimg_core::decode(&encoded).unwrap();
            (*name, decoded, *src_fmt)
        })
        .collect();

    let mut group = c.benchmark_group("convert");
    for (i, (name, src_image, _)) in sources.iter().enumerate() {
        let target_fmt = conversions[i].2;
        let pipeline_opts = PipelineOptions {
            format: target_fmt,
            quality: 80,
            resize: None,
        };
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            src_image,
            |b, image| {
                b.iter(|| convert(image, &pipeline_opts).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_optimize(c: &mut Criterion) {
    let image = generate_test_image(512, 512);
    let options = EncodeOptions { quality: 90 };

    let formats = vec![Format::Jpeg, Format::Png, Format::WebP];

    let mut group = c.benchmark_group("optimize");
    for format in formats {
        let codec = get_codec(format);
        let encoded = codec.encode(&image, &options).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", format)),
            &encoded,
            |b, data| {
                b.iter(|| optimize(data, 80).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_resize(c: &mut Criterion) {
    let image = generate_test_image(512, 512);

    let modes: Vec<(&str, ResizeMode)> = vec![
        ("width_256", ResizeMode::Width(256)),
        ("height_256", ResizeMode::Height(256)),
        ("scale_0.5", ResizeMode::Scale(0.5)),
        ("fit_256x256", ResizeMode::Fit(256, 256)),
    ];

    let mut group = c.benchmark_group("resize");
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
```

**Step 2: Run the pipeline benchmarks to verify they work**

Run: `cargo bench -p slimg-core --bench pipeline_bench`
Expected: Benchmark results for convert, optimize, resize groups

**Step 3: Commit**

```bash
git add crates/slimg-core/benches/pipeline_bench.rs
git commit -m "feat: add pipeline convert/optimize/resize benchmarks"
```

---

### Task 4: Run full benchmark suite and verify

**Step 1: Run all benchmarks**

Run: `cargo bench -p slimg-core`
Expected: All encode, decode, convert, optimize, resize benchmarks pass

**Step 2: Verify HTML report generation**

Run: `ls target/criterion/`
Expected: Directories for each benchmark group (encode, decode, convert, optimize, resize)

**Step 3: Final commit (if any fixes needed)**

Fix any compilation or runtime issues discovered during the full run.
