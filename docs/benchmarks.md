# Benchmarks

Performance measurements on 512x512 gradient test images using [criterion](https://github.com/bheisler/criterion.rs).

> Environment: Apple M-series, macOS, Rust release profile with thin LTO.
> Results will vary by hardware. Run `cargo bench -p slimg-core` to measure on your machine.

## Codec Performance

### Encode (512x512, quality 80)

| Format | Time | Throughput |
|--------|------|-----------|
| QOI | 352 µs | 744 Mpx/s |
| JPEG | 7.2 ms | 36 Mpx/s |
| WebP | 9.3 ms | 28 Mpx/s |
| AVIF | 29 ms | 9 Mpx/s |
| PNG | 45 ms | 5.8 Mpx/s |

### Decode

| Format | Time | Throughput |
|--------|------|-----------|
| QOI | 254 µs | 980 MiB/s |
| JPEG | 380 µs | 12.7 MiB/s |
| PNG | 676 µs | 2.8 MiB/s |
| AVIF | 945 µs | 3.4 MiB/s |
| WebP | 2.1 ms | 1.2 MiB/s |

## Pipeline Performance

### Format Conversion

| Pipeline | Time | Throughput |
|----------|------|-----------|
| PNG → JPEG | 7.8 ms | 33.5 Mpx/s |
| JPEG → WebP | 8.3 ms | 31.6 Mpx/s |
| PNG → AVIF | 28.6 ms | 9.2 Mpx/s |
| WebP → PNG | 203 ms | 1.3 Mpx/s |

### Optimize (re-encode same format)

| Format | Time |
|--------|------|
| WebP | 7.2 ms |
| JPEG | 8.4 ms |
| AVIF | 26 ms |
| PNG | 44 ms |

### Resize (512x512 → 256x256)

| Mode | Time | Throughput |
|------|------|-----------|
| Width | 2.09 ms | 125 Mpx/s |
| Height | 2.09 ms | 125 Mpx/s |
| Exact | 2.09 ms | 125 Mpx/s |
| Scale (0.5x) | 2.08 ms | 126 Mpx/s |
| Fit | 2.09 ms | 125 Mpx/s |

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench -p slimg-core

# Run specific group
cargo bench -p slimg-core -- encode
cargo bench -p slimg-core -- decode
cargo bench -p slimg-core -- convert
cargo bench -p slimg-core -- optimize
cargo bench -p slimg-core -- resize
```

HTML reports are generated at `target/criterion/report/index.html`.
