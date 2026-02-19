import { useState, useCallback, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  api,
  type ProcessOptions,
  type ProcessResult,
  type BatchProgress,
} from "@/lib/tauri";

export interface BatchItem {
  path: string;
  status: "pending" | "processing" | "completed" | "error";
  result?: ProcessResult;
  error?: string;
}

interface UseBatchProcessReturn {
  batchItems: BatchItem[];
  isProcessing: boolean;
  progress: number;
  processBatch: (paths: string[], options: ProcessOptions) => Promise<void>;
  reset: () => void;
}

export function useBatchProcess(): UseBatchProcessReturn {
  const [batchItems, setBatchItems] = useState<BatchItem[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  const completedCount = batchItems.filter(
    (item) => item.status === "completed" || item.status === "error"
  ).length;
  const progress =
    batchItems.length > 0
      ? Math.round((completedCount / batchItems.length) * 100)
      : 0;

  const processBatch = useCallback(
    async (paths: string[], options: ProcessOptions) => {
      const initialItems: BatchItem[] = paths.map((path) => ({
        path,
        status: "pending",
      }));
      setBatchItems(initialItems);
      setIsProcessing(true);

      const unlisten = await listen<BatchProgress>(
        "batch-progress",
        (event) => {
          const payload = event.payload;

          setBatchItems((prev) => {
            const next = [...prev];
            const item = next[payload.index];
            if (!item) return prev;

            next[payload.index] = {
              ...item,
              status: payload.status,
              result: payload.result,
              error: payload.error,
            };
            return next;
          });
        }
      );
      unlistenRef.current = unlisten;

      try {
        await api.processBatch(paths, options);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setBatchItems((prev) =>
          prev.map((item) =>
            item.status === "pending" || item.status === "processing"
              ? { ...item, status: "error", error: message }
              : item
          )
        );
      } finally {
        unlisten();
        unlistenRef.current = null;
        setIsProcessing(false);
      }
    },
    []
  );

  const reset = useCallback(() => {
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
    setBatchItems([]);
    setIsProcessing(false);
  }, []);

  return { batchItems, isProcessing, progress, processBatch, reset };
}
