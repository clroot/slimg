# 사용 가이드

[English](./usage.md)

slimg은 **convert**, **optimize**, **resize**, **crop**, **extend** 다섯 가지 명령어를 제공합니다.

## convert

이미지를 다른 포맷으로 변환합니다.

```
slimg convert photo.jpg --format webp
```

| 옵션 | 설명 |
|------|------|
| `--format`, `-f` | 대상 포맷: `jpeg`, `png`, `webp`, `avif`, `qoi` |
| `--quality`, `-q` | 인코딩 품질 0-100 (기본값: 80) |
| `--output`, `-o` | 출력 경로 (파일 또는 디렉토리) |
| `--recursive` | 하위 디렉토리 포함 처리 |
| `--jobs`, `-j` | 병렬 작업 수 (기본값: 전체 코어) |
| `--overwrite` | 기존 파일 덮어쓰기 |

**예시:**

```bash
# JPEG를 WebP로 변환 (기본 품질 80)
slimg convert photo.jpg --format webp

# AVIF로 변환 (품질 60)
slimg convert photo.png --format avif --quality 60

# 디렉토리 내 모든 이미지 변환
slimg convert ./images --format webp --output ./output --recursive

# 병렬 작업 수를 4개로 제한
slimg convert ./images --format webp --recursive --jobs 4
```

## optimize

같은 포맷으로 재인코딩하여 파일 크기를 줄입니다.

```
slimg optimize photo.jpg
```

| 옵션 | 설명 |
|------|------|
| `--quality`, `-q` | 인코딩 품질 0-100 (기본값: 80) |
| `--output`, `-o` | 출력 경로 (파일 또는 디렉토리) |
| `--recursive` | 하위 디렉토리 포함 처리 |
| `--jobs`, `-j` | 병렬 작업 수 (기본값: 전체 코어) |
| `--overwrite` | 원본 파일 덮어쓰기 |

**예시:**

```bash
# JPEG 최적화 (품질 80)
slimg optimize photo.jpg

# 원본 파일 덮어쓰기
slimg optimize photo.jpg --overwrite

# 디렉토리 내 이미지 일괄 최적화
slimg optimize ./images --quality 70 --recursive

# 병렬 작업 수를 2개로 제한 (대용량 이미지에 유용)
slimg optimize ./images --recursive --jobs 2
```

## resize

이미지를 리사이즈합니다. 포맷 변환도 함께 가능합니다.

```
slimg resize photo.jpg --width 800
```

| 옵션 | 설명 |
|------|------|
| `--width` | 대상 너비 (픽셀) |
| `--height` | 대상 높이 (픽셀) |
| `--scale` | 배율 (예: `0.5`는 절반 크기) |
| `--format`, `-f` | 다른 포맷으로 변환 |
| `--quality`, `-q` | 인코딩 품질 0-100 (기본값: 80) |
| `--output`, `-o` | 출력 경로 (파일 또는 디렉토리) |
| `--recursive` | 하위 디렉토리 포함 처리 |
| `--jobs`, `-j` | 병렬 작업 수 (기본값: 전체 코어) |
| `--overwrite` | 기존 파일 덮어쓰기 |

`--width`와 `--height`를 모두 지정하면, 비율을 유지하면서 지정 영역 안에 맞춥니다.

**예시:**

```bash
# 너비 기준 리사이즈 (비율 유지)
slimg resize photo.jpg --width 800

# 높이 기준 리사이즈
slimg resize photo.jpg --height 600

# 지정 영역 안에 맞추기 (비율 유지)
slimg resize photo.jpg --width 800 --height 600

# 배율로 리사이즈
slimg resize photo.jpg --scale 0.5

# 리사이즈 + 포맷 변환
slimg resize photo.jpg --width 400 --format webp --output thumb.webp
```

## crop

이미지를 좌표 또는 비율로 크롭합니다. 포맷 변환도 함께 가능합니다.

```
slimg crop photo.jpg --region 100,50,800,600
```

| 옵션 | 설명 |
|------|------|
| `--region` | 크롭 영역: `x,y,너비,높이` (예: `100,50,800,600`) |
| `--aspect` | 비율 크롭: `너비:높이` (예: `16:9`, `1:1`), 중앙 기준 |
| `--format`, `-f` | 다른 포맷으로 변환 |
| `--quality`, `-q` | 인코딩 품질 0-100 (기본값: 80) |
| `--output`, `-o` | 출력 경로 (파일 또는 디렉토리) |
| `--recursive` | 하위 디렉토리 포함 처리 |
| `--jobs`, `-j` | 병렬 작업 수 (기본값: 전체 코어) |
| `--overwrite` | 기존 파일 덮어쓰기 |

`--region`과 `--aspect`는 동시에 사용할 수 없습니다. 둘 중 하나는 필수입니다.

**예시:**

```bash
# 좌표로 크롭 (x=100, y=50, 너비=800, 높이=600)
slimg crop photo.jpg --region 100,50,800,600

# 16:9 비율로 크롭 (중앙 기준)
slimg crop photo.jpg --aspect 16:9

# 정사각형으로 크롭 (1:1)
slimg crop photo.jpg --aspect 1:1

# 크롭 후 WebP로 변환
slimg crop photo.jpg --region 0,0,500,500 --format webp

# 디렉토리 내 모든 이미지 일괄 크롭
slimg crop ./images --aspect 16:9 --output ./cropped --recursive
```

## extend

이미지에 여백을 추가하여 목표 비율이나 크기로 확장합니다. 원본 이미지는 새 캔버스의 중앙에 배치됩니다.

```
slimg extend photo.jpg --aspect 1:1
```

| 옵션 | 설명 |
|------|------|
| `--aspect` | 목표 비율: `너비:높이` (예: `1:1`, `16:9`) |
| `--size` | 목표 캔버스 크기: `너비x높이` (예: `1920x1080`) |
| `--color` | 여백 색상 (hex, 예: `'#FF0000'`, `'000000'`). 기본값: 흰색 |
| `--transparent` | 투명 배경 사용 (PNG, WebP 등) |
| `--format`, `-f` | 다른 포맷으로 변환 |
| `--quality`, `-q` | 인코딩 품질 0-100 (기본값: 80) |
| `--output`, `-o` | 출력 경로 (파일 또는 디렉토리) |
| `--recursive` | 하위 디렉토리 포함 처리 |
| `--jobs`, `-j` | 병렬 작업 수 (기본값: 전체 코어) |
| `--overwrite` | 기존 파일 덮어쓰기 |

`--aspect`와 `--size`는 동시에 사용할 수 없습니다. 둘 중 하나는 필수입니다.
`--color`와 `--transparent`는 동시에 사용할 수 없습니다.

**예시:**

```bash
# 정사각형(1:1)으로 확장 (흰색 여백)
slimg extend photo.jpg --aspect 1:1

# 16:9 비율로 확장 (검정 여백)
slimg extend photo.jpg --aspect 16:9 --color '#000000'

# 정확한 크기로 확장 (투명 배경, PNG)
slimg extend photo.png --size 1920x1080 --transparent

# 확장 + 포맷 변환
slimg extend photo.jpg --aspect 1:1 --transparent --format png

# 디렉토리 내 모든 이미지 일괄 확장
slimg extend ./images --aspect 1:1 --output ./squared --recursive
```

## 배치 처리

`--recursive` 옵션으로 디렉토리를 처리할 때, slimg은 [rayon](https://github.com/rayon-rs/rayon)을 통해 모든 CPU 코어를 활용합니다. `--jobs` 옵션으로 병렬 수를 제한할 수 있습니다.

```bash
# 모든 코어 대신 4개 스레드만 사용
slimg convert ./images --format webp --recursive --jobs 4
```

**에러 처리** — 파일 처리 중 오류가 발생하면 해당 파일을 건너뛰고 나머지를 계속 처리합니다. 실패한 파일 목록은 마지막에 요약 출력됩니다.

**안전한 덮어쓰기** — `--overwrite` 사용 시, 임시 파일에 먼저 쓴 뒤 성공하면 이름을 변경합니다. 인코딩이 실패하면 원본 파일이 보존됩니다.

## 라이브러리 사용

핵심 기능은 라이브러리 크레이트(`slimg-core`)로도 사용할 수 있습니다:

```rust
use slimg_core::*;

// 이미지 파일 디코딩
let (image, format) = decode_file(Path::new("photo.jpg"))?;

// WebP로 변환 + 확장 (여백 추가로 1:1 비율 만들기)
let result = convert(&image, &PipelineOptions {
    format: Format::WebP,
    quality: 80,
    resize: None,
    crop: None,
    extend: Some(ExtendMode::AspectRatio { width: 1, height: 1 }),
    fill_color: Some(FillColor::Solid([255, 255, 255, 255])),
})?;

// 결과 저장
result.save(Path::new("photo.webp"))?;
```
