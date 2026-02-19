import { useState, useCallback } from "react";
import { api, type ProcessOptions, type ProcessResult } from "@/lib/tauri";

interface UseImageProcessReturn {
  processing: boolean;
  result: ProcessResult | null;
  error: string | null;
  processImage: (path: string, options: ProcessOptions) => Promise<void>;
  reset: () => void;
}

export function useImageProcess(): UseImageProcessReturn {
  const [processing, setProcessing] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const processImage = useCallback(
    async (path: string, options: ProcessOptions) => {
      setProcessing(true);
      setResult(null);
      setError(null);
      try {
        const res = await api.processImage(path, options);
        setResult(res);
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
    setError(null);
  }, []);

  return { processing, result, error, processImage, reset };
}
