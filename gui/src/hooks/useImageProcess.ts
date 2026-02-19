import { useState, useCallback } from "react";
import {
  api,
  type ImageInfo,
  type ProcessOptions,
  type ProcessResult,
} from "@/lib/tauri";

interface UseImageProcessReturn {
  processing: boolean;
  result: ProcessResult | null;
  resultInfo: ImageInfo | null;
  error: string | null;
  processImage: (path: string, options: ProcessOptions) => Promise<void>;
  reset: () => void;
}

export function useImageProcess(): UseImageProcessReturn {
  const [processing, setProcessing] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const [resultInfo, setResultInfo] = useState<ImageInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  const processImage = useCallback(
    async (path: string, options: ProcessOptions) => {
      setProcessing(true);
      setResult(null);
      setResultInfo(null);
      setError(null);
      try {
        const res = await api.processImage(path, options);
        setResult(res);

        // Load result file info (thumbnail) for before/after comparison
        try {
          const info = await api.loadImage(res.output_path);
          setResultInfo(info);
        } catch {
          // Result file info is optional; comparison view will show "Loading..." fallback
          console.warn("Failed to load result file info for preview");
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
      } finally {
        setProcessing(false);
      }
    },
    []
  );

  const reset = useCallback(() => {
    setResult(null);
    setResultInfo(null);
    setError(null);
  }, []);

  return { processing, result, resultInfo, error, processImage, reset };
}
