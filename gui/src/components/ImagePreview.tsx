import type { ImageInfo, ProcessResult } from "@/lib/tauri";
import { basename } from "@/lib/path";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

interface ImagePreviewProps {
  original: { path: string; info: ImageInfo };
  result: ProcessResult;
  resultInfo?: ImageInfo;
}

function ImagePanel({
  label,
  thumbnailBase64,
  fileName,
  sizeBytes,
  width,
  height,
  format,
}: {
  label: string;
  thumbnailBase64?: string;
  fileName: string;
  sizeBytes: number;
  width: number;
  height: number;
  format: string;
}) {
  return (
    <div className="flex flex-1 flex-col overflow-hidden rounded-xl border bg-card">
      <div className="border-b px-4 py-2">
        <span className="text-sm font-bold">{label}</span>
      </div>
      <div className="flex aspect-video items-center justify-center bg-muted">
        {thumbnailBase64 ? (
          <img
            src={`data:image/png;base64,${thumbnailBase64}`}
            alt={`${label} - ${fileName}`}
            className="max-h-full max-w-full object-contain"
          />
        ) : (
          <div className="flex items-center justify-center text-sm text-muted-foreground">
            Loading...
          </div>
        )}
      </div>
      <div className="space-y-1 p-4">
        <p className="truncate text-sm font-medium" title={fileName}>
          {fileName}
        </p>
        <p className="text-xs text-muted-foreground">
          {width} x {height} &middot; {format.toUpperCase()} &middot;{" "}
          {formatBytes(sizeBytes)}
        </p>
      </div>
    </div>
  );
}

function CompressionBadge({
  originalSize,
  newSize,
}: {
  originalSize: number;
  newSize: number;
}) {
  const savingsPercent =
    originalSize > 0 ? (1 - newSize / originalSize) * 100 : 0;
  const isSizeReduced = savingsPercent > 0;

  return (
    <div className="flex flex-col items-center justify-center gap-1">
      <span className="text-xs font-medium text-muted-foreground">
        {isSizeReduced ? "Saved" : "Change"}
      </span>
      <span
        className={`text-lg font-bold ${
          isSizeReduced
            ? "text-green-600 dark:text-green-400"
            : "text-amber-600 dark:text-amber-400"
        }`}
      >
        {isSizeReduced ? "-" : "+"}
        {Math.abs(savingsPercent).toFixed(1)}%
      </span>
    </div>
  );
}

export function ImagePreview({
  original,
  result,
  resultInfo,
}: ImagePreviewProps) {
  return (
    <div className="flex items-stretch gap-4">
      <ImagePanel
        label="Before"
        thumbnailBase64={original.info.thumbnail_base64}
        fileName={basename(original.path)}
        sizeBytes={original.info.size_bytes}
        width={original.info.width}
        height={original.info.height}
        format={original.info.format}
      />

      <CompressionBadge
        originalSize={result.original_size}
        newSize={result.new_size}
      />

      <ImagePanel
        label="After"
        thumbnailBase64={resultInfo?.thumbnail_base64}
        fileName={basename(result.output_path)}
        sizeBytes={result.new_size}
        width={result.width}
        height={result.height}
        format={result.format}
      />
    </div>
  );
}
