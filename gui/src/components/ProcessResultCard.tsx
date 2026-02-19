import type { ProcessResult } from "@/lib/tauri";
import { basename } from "@/lib/path";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function ProcessResultCard({ result }: { result: ProcessResult }) {
  const savingsPercent =
    result.original_size > 0
      ? (1 - result.new_size / result.original_size) * 100
      : 0;
  const isSizeReduced = savingsPercent > 0;

  return (
    <div className="space-y-3 rounded-lg border bg-card p-4">
      <p className="text-sm font-medium">Result</p>
      <div className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-muted-foreground">Output</span>
          <span
            className="ml-4 max-w-60 truncate text-right"
            title={result.output_path}
          >
            {basename(result.output_path)}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">Size</span>
          <span>
            {formatBytes(result.original_size)} &rarr;{" "}
            {formatBytes(result.new_size)}{" "}
            <span
              className={
                isSizeReduced
                  ? "text-green-600 dark:text-green-400"
                  : "text-amber-600 dark:text-amber-400"
              }
            >
              ({isSizeReduced ? "-" : "+"}
              {Math.abs(savingsPercent).toFixed(1)}%)
            </span>
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">Resolution</span>
          <span>
            {result.width} x {result.height}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">Format</span>
          <span>{result.format.toUpperCase()}</span>
        </div>
      </div>
    </div>
  );
}
