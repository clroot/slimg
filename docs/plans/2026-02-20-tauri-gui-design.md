# Tauri GUI Design for slimg

## Overview

slimg CLI의 전체 기능을 비개발자/디자이너가 쉽게 사용할 수 있는 데스크톱 GUI 애플리케이션으로 제공한다.
Tauri v2 + React + Tailwind + shadcn/ui 기반으로, 기존 `slimg-core` crate를 직접 참조한다.

## Target Users

비개발자/디자이너. CLI에 익숙하지 않지만 이미지 포맷 변환, 압축, 리사이즈 등이 필요한 사용자.

## Core Features

5개 기능 전체 지원:
- **Convert**: 포맷 변환 (JPEG, PNG, WebP, AVIF, QOI, JXL)
- **Optimize**: 동일 포맷 재압축
- **Resize**: 크기 조절 (비율 유지 옵션)
- **Crop**: 영역/비율 기반 자르기
- **Extend**: 패딩 추가 (비율 맞추기)

단일 파일 및 배치 처리 모두 지원.

## UI Layout

사이드바 네비게이션 + 작업 영역 구조.

```
┌──────────────────────────────────────────────────┐
│  slimg                              [─] [□] [✕]  │
├────────────┬─────────────────────────────────────┤
│            │                                     │
│  Convert   │   ┌─────────────────────────────┐   │
│  Optimize  │   │                             │   │
│  Resize    │   │    Drop zone / Preview      │   │
│  Crop      │   │                             │   │
│  Extend    │   └─────────────────────────────┘   │
│            │                                     │
│            │   ┌─────────────────────────────┐   │
│            │   │  Options Panel               │   │
│            │   │  (format, quality, etc.)     │   │
│            │   └─────────────────────────────┘   │
│            │                                     │
│────────────│   [ Process ]                       │
│  Settings  │                                     │
└────────────┴─────────────────────────────────────┘
```

### Sidebar (~200px)

- 5개 기능 아이콘 + 라벨
- 하단에 Settings (출력 디렉토리 기본값 등)

### Work Area

- 상단: 파일 드롭존 / 이미지 미리보기 (전/후 비교)
- 중단: 선택한 기능에 따른 옵션 패널
- 하단: Process 버튼 + 진행 상태

## Options Panel per Feature

### Convert

- 포맷 드롭다운: JPEG, PNG, WebP, AVIF, QOI, JXL
- 품질 슬라이더: 0~100 (기본값 80)

### Optimize

- 품질 슬라이더만. 포맷은 원본 유지.

### Resize

- Width/Height 입력 (한쪽 입력 시 비율 자동 계산)
- 비율 유지 토글
- 선택적 포맷 변환 + 품질

### Crop

- 모드 전환: 좌표 지정 (x, y, w, h) / 비율 지정 (w:h)
- 선택적 포맷 변환 + 품질

### Extend

- 목표 비율 입력 (w:h)
- 채움 색상 컬러 피커
- 선택적 포맷 변환 + 품질

## Preview & Comparison

### Single File - Before/After

좌우 분할 비교 뷰:
- 원본 / 결과 나란히 표시
- 파일명, 크기, 해상도 표시
- 압축률(%) 표시

### Batch - File List + Progress

- 파일 목록에 각 파일 상태 표시 (대기/처리중/완료/에러)
- 완료된 파일 클릭 시 전/후 비교 뷰로 전환
- 전체 진행률 바

## File Input/Output

### Input

- 드래그앤드롭 (Tauri `onDragDropEvent`)
- 파일 선택 버튼 (Tauri `dialog.open`)
- 단일/복수 파일 및 폴더 지원

### Output

- 기본: 원본과 같은 디렉토리에 새 확장자로 저장
- 옵션: 출력 디렉토리 지정
- 옵션: 원본 덮어쓰기 (확인 다이얼로그 표시)

## Technical Architecture

### Tauri IPC Commands

```rust
// 이미지 디코딩 + 메타데이터 반환
#[tauri::command]
fn load_image(path: String) -> Result<ImageInfo, String>
// → { width, height, format, size_bytes, thumbnail_base64 }

// 단일 파일 처리
#[tauri::command]
fn process_image(input: String, options: ProcessOptions) -> Result<ProcessResult, String>
// → { output_path, size_bytes, format }

// 배치 처리 (이벤트로 진행률 전송)
#[tauri::command]
fn process_batch(inputs: Vec<String>, options: ProcessOptions, window: Window) -> Result<(), String>
// → window.emit("batch-progress", { index, total, status, result })

// 미리보기 생성 (인코딩만, 파일 저장 X)
#[tauri::command]
fn preview_image(input: String, options: ProcessOptions) -> Result<PreviewResult, String>
// → { data_base64, size_bytes, width, height }
```

### Dependencies

```toml
# gui/src-tauri/Cargo.toml
[dependencies]
slimg-core = { path = "../../crates/slimg-core" }
tauri = { version = "2", features = ["dialog"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
base64 = "0.22"
```

### Frontend Structure

```
gui/
├── src-tauri/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── commands.rs
├── src/
│   ├── App.tsx
│   ├── components/
│   │   ├── Sidebar.tsx
│   │   ├── DropZone.tsx
│   │   ├── ImagePreview.tsx
│   │   ├── BatchList.tsx
│   │   ├── OptionsPanel.tsx
│   │   └── options/
│   │       ├── ConvertOptions.tsx
│   │       ├── OptimizeOptions.tsx
│   │       ├── ResizeOptions.tsx
│   │       ├── CropOptions.tsx
│   │       └── ExtendOptions.tsx
│   ├── hooks/
│   │   ├── useImageProcess.ts
│   │   └── useBatchProcess.ts
│   └── lib/
│       └── tauri.ts
├── package.json
├── tailwind.config.ts
├── tsconfig.json
└── vite.config.ts
```

### Data Flow

```
User drops file
  → Frontend: invoke("load_image", path)
  → Backend: slimg_core::decode_file() → ImageInfo
  → Frontend: 썸네일 표시 + 옵션 패널 활성화

User clicks Process
  → Frontend: invoke("process_image", { path, options })
  → Backend: slimg_core::convert() → 파일 저장 → ProcessResult
  → Frontend: invoke("load_image", output_path) → 전/후 비교 표시

Batch mode
  → Frontend: invoke("process_batch", { paths, options })
  → Backend: 각 파일마다 window.emit("batch-progress", ...)
  → Frontend: 이벤트 리스너로 실시간 진행률 업데이트
```

## Tech Stack

- **Backend**: Tauri v2 + slimg-core (Rust)
- **Frontend**: React + TypeScript + Vite
- **Styling**: Tailwind CSS + shadcn/ui
- **Package Manager**: bun
