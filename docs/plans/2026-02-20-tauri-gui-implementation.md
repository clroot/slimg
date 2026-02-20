# Tauri GUI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** slimg CLI의 전체 기능을 Tauri v2 데스크톱 GUI로 제공한다.

**Architecture:** Tauri v2 backend가 slimg-core crate를 직접 참조하여 이미지 처리를 수행하고, React frontend가 IPC를 통해 Rust command를 호출한다. 사이드바 네비게이션 + 작업 영역 레이아웃.

**Tech Stack:** Tauri v2, React 19, TypeScript, Vite, Tailwind CSS v4, shadcn/ui, bun

**Design Doc:** `docs/plans/2026-02-20-tauri-gui-design.md`

---

## Task 1: Tauri 프로젝트 스캐폴딩

**Files:**
- Create: `gui/` (Tauri + React 프로젝트)
- Modify: `Cargo.toml` (workspace members에 gui/src-tauri 추가)

**Step 1: Tauri CLI로 프로젝트 생성**

```bash
cd /Users/clroot/workspace/io.clroot/slimg
bun create tauri-app gui --template react-ts --manager bun --yes
```

생성되지 않거나 옵션이 다르면 Tauri v2 공식 문서를 참조해서 수동으로 생성한다.

**Step 2: Cargo workspace에 gui/src-tauri 추가**

`Cargo.toml` workspace members에 `"gui/src-tauri"` 추가:

```toml
[workspace]
resolver = "2"
members = ["crates/slimg-core", "crates/slimg-ffi", "crates/libjxl-sys", "cli", "gui/src-tauri"]
```

**Step 3: gui/src-tauri/Cargo.toml에 slimg-core 의존성 추가**

```toml
[dependencies]
slimg-core = { path = "../../crates/slimg-core" }
```

**Step 4: 기본 Tauri 앱이 빌드되고 실행되는지 확인**

```bash
cd gui && bun run tauri dev
```

Expected: 빈 Tauri 윈도우가 열림

**Step 5: Commit**

```bash
git add gui/ Cargo.toml Cargo.lock
git commit -m "feat(gui): scaffold Tauri v2 + React project"
```

---

## Task 2: Tailwind CSS v4 + shadcn/ui 설정

**Files:**
- Modify: `gui/package.json`
- Modify: `gui/src/index.css` (또는 App.css)
- Create: `gui/components.json` (shadcn/ui 설정)

**Step 1: Tailwind CSS v4 설치**

```bash
cd gui && bun add -D tailwindcss @tailwindcss/vite
```

Vite 플러그인 설정 (`vite.config.ts`):
```ts
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
});
```

CSS 파일에 `@import "tailwindcss"` 추가.

**Step 2: shadcn/ui 초기화**

```bash
cd gui && bunx shadcn@latest init
```

프롬프트에서: New York 스타일, Zinc 색상, CSS variables 사용.

**Step 3: 필요한 shadcn 컴포넌트 설치**

```bash
cd gui && bunx shadcn@latest add button select slider input label radio-group checkbox tabs progress separator tooltip
```

**Step 4: 빌드 확인**

```bash
cd gui && bun run tauri dev
```

Expected: Tailwind 스타일이 적용된 Tauri 앱

**Step 5: Commit**

```bash
git add gui/
git commit -m "feat(gui): setup Tailwind CSS v4 and shadcn/ui"
```

---

## Task 3: Tauri Rust Backend - IPC Commands

**Files:**
- Create: `gui/src-tauri/src/commands.rs`
- Modify: `gui/src-tauri/src/lib.rs` (또는 `main.rs`)

**Step 1: commands.rs에 타입 정의**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub size_bytes: u64,
    pub thumbnail_base64: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessOptions {
    pub operation: String,        // "convert" | "optimize" | "resize" | "crop" | "extend"
    pub format: Option<String>,   // 출력 포맷 (convert, resize, crop, extend)
    pub quality: u8,              // 0-100
    pub width: Option<u32>,       // resize, crop region, extend aspect
    pub height: Option<u32>,
    pub x: Option<u32>,           // crop region
    pub y: Option<u32>,
    pub crop_mode: Option<String>, // "region" | "aspect"
    pub fill_color: Option<String>, // hex color (extend)
    pub resize_mode: Option<String>, // "width" | "height" | "exact" | "fit"
    pub output_dir: Option<String>,
    pub overwrite: bool,
}

#[derive(Debug, Serialize)]
pub struct ProcessResult {
    pub output_path: String,
    pub original_size: u64,
    pub new_size: u64,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct PreviewResult {
    pub data_base64: String,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchProgress {
    pub index: usize,
    pub total: usize,
    pub file_path: String,
    pub status: String,          // "processing" | "completed" | "error"
    pub result: Option<ProcessResult>,
    pub error: Option<String>,
}
```

**Step 2: load_image command 구현**

`load_image`는 파일을 디코딩하고 메타데이터 + base64 썸네일을 반환한다.

- `slimg_core::decode_file()`로 디코딩
- 썸네일: 디코딩된 ImageData를 최대 400px로 리사이즈
- `image` crate로 RGBA → PNG 인코딩 → base64
- 파일 크기: `std::fs::metadata`

```rust
#[tauri::command]
pub fn load_image(path: String) -> Result<ImageInfo, String> {
    // 구현: decode_file → 메타데이터 추출 → 썸네일 생성 → ImageInfo 반환
}
```

**Step 3: process_image command 구현**

ProcessOptions를 slimg-core의 PipelineOptions로 변환하여 처리한다.

- operation에 따라 적절한 PipelineOptions 구성
- `slimg_core::convert()` 또는 `slimg_core::optimize()` 호출
- `output_path()` 로 출력 경로 결정 → 파일 저장

```rust
#[tauri::command]
pub fn process_image(input: String, options: ProcessOptions) -> Result<ProcessResult, String> {
    // 구현: options → PipelineOptions → convert/optimize → 저장 → ProcessResult
}
```

**Step 4: preview_image command 구현**

process_image와 동일하지만 파일 저장 없이 base64로 결과 반환.

```rust
#[tauri::command]
pub fn preview_image(input: String, options: ProcessOptions) -> Result<PreviewResult, String> {
    // 구현: options → PipelineOptions → convert/optimize → base64 인코딩 → PreviewResult
}
```

**Step 5: process_batch command 구현**

여러 파일을 순회하며 처리하고, 각 파일마다 Tauri 이벤트로 진행률 전송.

```rust
#[tauri::command]
pub fn process_batch(
    inputs: Vec<String>,
    options: ProcessOptions,
    window: tauri::Window,
) -> Result<(), String> {
    // 구현: inputs 순회 → 각 파일 처리 → window.emit("batch-progress", BatchProgress)
}
```

**Step 6: main.rs에 commands 등록**

```rust
mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::load_image,
            commands::process_image,
            commands::preview_image,
            commands::process_batch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 7: 빌드 확인**

```bash
cd gui && bun run tauri dev
```

Expected: 컴파일 성공 (프론트엔드에서 아직 호출하지 않으므로 UI 변화 없음)

**Step 8: Commit**

```bash
git add gui/src-tauri/
git commit -m "feat(gui): implement Tauri IPC commands for image processing"
```

---

## Task 4: Sidebar 네비게이션 컴포넌트

**Files:**
- Create: `gui/src/components/Sidebar.tsx`
- Modify: `gui/src/App.tsx`

**Step 1: Sidebar 컴포넌트 구현**

5개 기능 아이콘 + 라벨, 하단에 Settings.
lucide-react 아이콘 사용.

```bash
cd gui && bun add lucide-react
```

- Convert: `ArrowRightLeft`
- Optimize: `Zap`
- Resize: `Maximize2`
- Crop: `Crop`
- Extend: `Expand`
- Settings: `Settings`

사이드바 너비 200px, 고정. 선택된 항목에 활성 스타일.

**Step 2: App.tsx에 레이아웃 구성**

```tsx
function App() {
  const [activeFeature, setActiveFeature] = useState<Feature>("convert");
  return (
    <div className="flex h-screen">
      <Sidebar active={activeFeature} onSelect={setActiveFeature} />
      <main className="flex-1 overflow-auto p-6">
        {/* 작업 영역 - 이후 Task에서 구현 */}
      </main>
    </div>
  );
}
```

**Step 3: 실행 확인**

```bash
cd gui && bun run tauri dev
```

Expected: 좌측에 사이드바, 우측에 빈 작업 영역

**Step 4: Commit**

```bash
git add gui/src/
git commit -m "feat(gui): add sidebar navigation component"
```

---

## Task 5: DropZone 파일 입력 컴포넌트

**Files:**
- Create: `gui/src/components/DropZone.tsx`
- Create: `gui/src/lib/tauri.ts`

**Step 1: Tauri invoke wrapper 작성**

`gui/src/lib/tauri.ts`에 타입 안전한 invoke wrapper 작성.

```ts
import { invoke } from "@tauri-apps/api/core";

export interface ImageInfo { ... }
export interface ProcessOptions { ... }
export interface ProcessResult { ... }
export interface PreviewResult { ... }
export interface BatchProgress { ... }

export const api = {
  loadImage: (path: string) => invoke<ImageInfo>("load_image", { path }),
  processImage: (input: string, options: ProcessOptions) => invoke<ProcessResult>("process_image", { input, options }),
  previewImage: (input: string, options: ProcessOptions) => invoke<PreviewResult>("preview_image", { input, options }),
  processBatch: (inputs: string[], options: ProcessOptions) => invoke<void>("process_batch", { inputs, options }),
};
```

**Step 2: DropZone 컴포넌트 구현**

- Tauri의 drag-and-drop 이벤트(`onDragDropEvent`) 또는 `tauri-plugin-dialog`의 파일 선택 사용
- 드롭 시 파일 경로를 받아 `load_image` 호출
- 단일/복수 파일 모두 지원
- 드래그 중 시각적 피드백 (border 색상 변경)
- 파일 선택 버튼도 제공

```bash
cd gui && bun add @tauri-apps/plugin-dialog
```

Tauri 플러그인도 등록:
```rust
// src-tauri/src/main.rs
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
```

```toml
# src-tauri/Cargo.toml
tauri-plugin-dialog = "2"
```

**Step 3: App.tsx에 DropZone 연결**

파일이 선택되면 상태에 저장하고, 썸네일 표시.

**Step 4: 실행 확인**

```bash
cd gui && bun run tauri dev
```

Expected: 파일을 드래그하면 드롭존 활성화, 파일 선택 시 load_image 호출

**Step 5: Commit**

```bash
git add gui/
git commit -m "feat(gui): add DropZone file input component"
```

---

## Task 6: 기능별 Options 패널 컴포넌트

**Files:**
- Create: `gui/src/components/options/ConvertOptions.tsx`
- Create: `gui/src/components/options/OptimizeOptions.tsx`
- Create: `gui/src/components/options/ResizeOptions.tsx`
- Create: `gui/src/components/options/CropOptions.tsx`
- Create: `gui/src/components/options/ExtendOptions.tsx`
- Create: `gui/src/components/OptionsPanel.tsx`

**Step 1: ConvertOptions 구현**

- 포맷 Select 드롭다운 (JPEG, PNG, WebP, AVIF, QOI, JXL)
- Quality Slider (0~100, 기본 80)
- onChange로 부모에 옵션 전달

**Step 2: OptimizeOptions 구현**

- Quality Slider만

**Step 3: ResizeOptions 구현**

- Width/Height 숫자 Input
- 비율 유지 Checkbox (기본 on)
- 비율 유지 시 한쪽 입력하면 다른쪽 자동 계산 (원본 이미지 비율 기반)
- 선택적 포맷 Select + Quality Slider

**Step 4: CropOptions 구현**

- 모드 RadioGroup: Region / Aspect Ratio
- Region: x, y, width, height 숫자 Input
- Aspect: width, height 숫자 Input
- 선택적 포맷 Select + Quality Slider

**Step 5: ExtendOptions 구현**

- Aspect Ratio: width, height 숫자 Input
- Fill Color: 텍스트 Input (hex) + 색상 미리보기
- 선택적 포맷 Select + Quality Slider

**Step 6: OptionsPanel - 기능별 라우팅**

```tsx
function OptionsPanel({ feature, imageInfo, onChange }) {
  switch (feature) {
    case "convert": return <ConvertOptions ... />;
    case "optimize": return <OptimizeOptions ... />;
    case "resize": return <ResizeOptions ... />;
    case "crop": return <CropOptions ... />;
    case "extend": return <ExtendOptions ... />;
  }
}
```

**Step 7: App.tsx에 OptionsPanel 연결**

사이드바 선택에 따라 옵션 패널 전환.

**Step 8: 실행 확인**

```bash
cd gui && bun run tauri dev
```

Expected: 사이드바 기능 전환 시 옵션 패널이 바뀜

**Step 9: Commit**

```bash
git add gui/src/
git commit -m "feat(gui): add feature-specific options panels"
```

---

## Task 7: 이미지 처리 실행 + 결과 표시

**Files:**
- Create: `gui/src/hooks/useImageProcess.ts`
- Modify: `gui/src/App.tsx`

**Step 1: useImageProcess 훅 구현**

```ts
function useImageProcess() {
  // 상태: processing, result, error
  // processImage(path, options) → api.processImage 호출
  // previewImage(path, options) → api.previewImage 호출
  // 로딩 상태 관리
}
```

**Step 2: Process 버튼 추가**

App.tsx 하단에 "Process" 버튼. 클릭 시 현재 파일 + 옵션으로 processImage 호출.

**Step 3: 결과 표시**

처리 완료 시:
- 출력 파일 경로 표시
- 원본/결과 크기 비교
- 압축률 표시

**Step 4: 실행 확인**

```bash
cd gui && bun run tauri dev
```

Expected: 파일 선택 → 옵션 설정 → Process 클릭 → 결과 표시

**Step 5: Commit**

```bash
git add gui/src/
git commit -m "feat(gui): add image processing execution and result display"
```

---

## Task 8: 전/후 비교 미리보기 뷰

**Files:**
- Create: `gui/src/components/ImagePreview.tsx`
- Modify: `gui/src/App.tsx`

**Step 1: ImagePreview 컴포넌트 구현**

좌우 분할 비교 뷰:
- Before (원본): `load_image`에서 받은 thumbnail_base64
- After (결과): `process_image` 후 결과 파일을 `load_image`로 로드
- 각 이미지 아래에 파일명, 크기, 해상도 표시
- 압축률(%) 표시

이미지 표시는 Tauri의 `convertFileSrc`로 로컬 파일을 asset URL로 변환하거나, base64 data URL 사용.

**Step 2: DropZone ↔ ImagePreview 전환**

- 파일 미선택: DropZone 표시
- 파일 선택됨: 원본 썸네일 표시
- 처리 완료: 전/후 비교 뷰 표시

**Step 3: 실행 확인**

```bash
cd gui && bun run tauri dev
```

Expected: 처리 후 원본과 결과를 나란히 비교

**Step 4: Commit**

```bash
git add gui/src/
git commit -m "feat(gui): add before/after image comparison preview"
```

---

## Task 9: 배치 처리 + 파일 목록

**Files:**
- Create: `gui/src/hooks/useBatchProcess.ts`
- Create: `gui/src/components/BatchList.tsx`
- Modify: `gui/src/App.tsx`

**Step 1: useBatchProcess 훅 구현**

```ts
function useBatchProcess() {
  // Tauri 이벤트 "batch-progress" 리스너
  // 파일별 상태 추적: pending | processing | completed | error
  // 전체 진행률 계산
  // processBatch(paths, options) 호출
}
```

`@tauri-apps/api/event`의 `listen` 사용:
```ts
import { listen } from "@tauri-apps/api/event";
```

**Step 2: BatchList 컴포넌트 구현**

- 파일 목록 표시 (상태 아이콘 + 파일명 + 결과)
- 전체 Progress 바
- Clear 버튼
- 완료된 파일 클릭 시 해당 파일의 전/후 비교 표시

**Step 3: App.tsx에서 단일/배치 모드 분기**

- 파일 1개: 단일 모드 (기존 흐름)
- 파일 2개 이상: 배치 모드 (BatchList 표시)

**Step 4: 실행 확인**

```bash
cd gui && bun run tauri dev
```

Expected: 여러 파일 드롭 → 배치 처리 → 실시간 진행률 → 완료 후 개별 비교 가능

**Step 5: Commit**

```bash
git add gui/src/
git commit -m "feat(gui): add batch processing with progress tracking"
```

---

## Task 10: Settings + 마무리

**Files:**
- Create: `gui/src/components/Settings.tsx`
- Modify: `gui/src/components/Sidebar.tsx`
- Modify: `gui/src/App.tsx`

**Step 1: Settings 패널 구현**

- 기본 출력 디렉토리 설정
- 기본 품질 값 설정
- 원본 덮어쓰기 기본값

설정은 localStorage에 저장 (간단한 MVP).

**Step 2: Sidebar에 Settings 연결**

사이드바 하단 Settings 클릭 시 Settings 패널 표시.

**Step 3: 윈도우 설정 조정**

`gui/src-tauri/tauri.conf.json`에서:
- 타이틀: "slimg"
- 기본 크기: 1024x768
- 최소 크기: 800x600

**Step 4: 전체 동작 확인**

```bash
cd gui && bun run tauri dev
```

체크리스트:
- [ ] 5개 기능 모두 동작
- [ ] 단일 파일 처리 + 미리보기
- [ ] 배치 파일 처리 + 진행률
- [ ] 전/후 비교 뷰
- [ ] Settings 저장/로드

**Step 5: Commit**

```bash
git add gui/
git commit -m "feat(gui): add settings panel and finalize UI"
```
