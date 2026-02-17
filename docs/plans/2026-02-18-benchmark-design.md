# Benchmark Design

## Overview

slimg-core에 criterion 기반 벤치마크를 추가하여 코덱별 encode/decode 성능과 파이프라인 전체 성능을 측정한다.

## Structure

```
crates/slimg-core/
├── benches/
│   ├── codec_bench.rs      # 코덱별 encode/decode
│   └── pipeline_bench.rs   # 파이프라인(convert, optimize, resize)
```

## Benchmarks

### codec_bench.rs

| Benchmark | Description |
|-----------|-------------|
| `encode/{format}` | 각 포맷의 인코딩 속도 (JPEG, PNG, WebP, AVIF, QOI) |
| `decode/{format}` | 각 포맷의 디코딩 속도 |

- 테스트 이미지: 512x512 그라데이션 RGBA 이미지 (코드 생성)
- quality 80 고정

### pipeline_bench.rs

| Benchmark | Description |
|-----------|-------------|
| `convert/jpeg_to_webp` | JPEG → WebP 변환 파이프라인 |
| `convert/png_to_avif` | PNG → AVIF 변환 파이프라인 |
| `optimize/{format}` | 같은 포맷 재인코딩 최적화 |
| `resize/{mode}` | Width, Scale 리사이즈 모드별 측정 |

## Test Data

`image` crate로 512x512 그라데이션 패턴 이미지를 코드로 생성한다. 외부 파일 의존 없이 CI에서도 바로 동작.

## Usage

```bash
cargo bench -p slimg-core              # 전체
cargo bench -p slimg-core -- encode    # encode만
cargo bench -p slimg-core -- decode    # decode만
cargo bench -p slimg-core -- resize    # resize만
```

criterion이 `target/criterion/`에 HTML 리포트를 자동 생성하고 이전 실행 대비 성능 변화를 추적한다.
