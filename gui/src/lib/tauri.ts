import { invoke } from "@tauri-apps/api/core";

export interface ImageInfo {
  width: number;
  height: number;
  format: string;
  size_bytes: number;
  thumbnail_base64: string;
}

export interface ProcessOptions {
  operation: "convert" | "optimize" | "resize" | "crop" | "extend";
  format?: string;
  quality: number;
  width?: number;
  height?: number;
  x?: number;
  y?: number;
  crop_mode?: "region" | "aspect";
  fill_color?: string;
  resize_mode?: "width" | "height" | "exact" | "fit";
  output_dir?: string;
  overwrite: boolean;
}

export interface ProcessResult {
  output_path: string;
  original_size: number;
  new_size: number;
  width: number;
  height: number;
  format: string;
}

export interface PreviewResult {
  data_base64: string;
  size_bytes: number;
  width: number;
  height: number;
  format: string;
}

export interface BatchProgress {
  index: number;
  total: number;
  file_path: string;
  status: "processing" | "completed" | "error";
  result?: ProcessResult;
  error?: string;
}

export const api = {
  scanDirectory: (path: string) => invoke<string[]>("scan_directory", { path }),
  loadImage: (path: string) => invoke<ImageInfo>("load_image", { path }),
  processImage: (input: string, options: ProcessOptions) =>
    invoke<ProcessResult>("process_image", { input, options }),
  previewImage: (input: string, options: ProcessOptions) =>
    invoke<PreviewResult>("preview_image", { input, options }),
  processBatch: (inputs: string[], options: ProcessOptions) =>
    invoke<void>("process_batch", { inputs, options }),
};
