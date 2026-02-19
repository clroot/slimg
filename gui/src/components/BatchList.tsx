import { CheckCircle2, XCircle, Loader2, Circle } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import { formatBytes } from "@/lib/format";
import { basename } from "@/lib/path";
import type { BatchItem } from "@/hooks/useBatchProcess";
import { cn } from "@/lib/utils";

interface BatchListProps {
  items: BatchItem[];
  progress: number;
  onItemClick?: (index: number) => void;
  selectedIndex?: number;
}

function StatusIcon({ status }: { status: BatchItem["status"] }) {
  switch (status) {
    case "pending":
      return <Circle className="h-4 w-4 text-muted-foreground" />;
    case "processing":
      return <Loader2 className="h-4 w-4 animate-spin text-primary" />;
    case "completed":
      return (
        <CheckCircle2 className="h-4 w-4 text-green-600 dark:text-green-400" />
      );
    case "error":
      return <XCircle className="h-4 w-4 text-destructive" />;
  }
}

function SizeChange({
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
    <span className="text-xs text-muted-foreground">
      {formatBytes(originalSize)} &rarr; {formatBytes(newSize)}{" "}
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
  );
}

export function BatchList({
  items,
  progress,
  onItemClick,
  selectedIndex,
}: BatchListProps) {
  const completedCount = items.filter(
    (item) => item.status === "completed"
  ).length;
  const errorCount = items.filter((item) => item.status === "error").length;

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <div className="flex items-center justify-between text-sm">
          <span className="font-medium">
            {progress < 100 ? "Processing..." : "Complete"}
          </span>
          <span className="text-muted-foreground">
            {completedCount + errorCount} / {items.length}
          </span>
        </div>
        <Progress value={progress} />
      </div>

      <div className="space-y-1 rounded-lg border bg-card">
        {items.map((item, index) => {
          const isClickable =
            item.status === "completed" && onItemClick !== undefined;

          return (
            <button
              key={item.path}
              type="button"
              disabled={!isClickable}
              onClick={() => isClickable && onItemClick(index)}
              className={cn(
                "flex w-full items-center gap-3 px-4 py-3 text-left transition-colors",
                "disabled:cursor-default",
                index !== items.length - 1 && "border-b",
                isClickable && "hover:bg-accent",
                selectedIndex === index && "bg-accent"
              )}
            >
              <StatusIcon status={item.status} />

              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">
                  {basename(item.path)}
                </p>
                {item.status === "completed" && item.result && (
                  <SizeChange
                    originalSize={item.result.original_size}
                    newSize={item.result.new_size}
                  />
                )}
                {item.status === "error" && item.error && (
                  <p className="truncate text-xs text-destructive">
                    {item.error}
                  </p>
                )}
              </div>

              {item.status === "completed" && item.result && (
                <span className="shrink-0 text-xs text-muted-foreground">
                  {item.result.format.toUpperCase()}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {progress === 100 && (
        <div className="rounded-lg border bg-card p-4">
          <p className="text-sm font-medium">Summary</p>
          <div className="mt-2 flex gap-6 text-sm">
            <span className="text-muted-foreground">
              Completed:{" "}
              <span className="font-medium text-green-600 dark:text-green-400">
                {completedCount}
              </span>
            </span>
            {errorCount > 0 && (
              <span className="text-muted-foreground">
                Errors:{" "}
                <span className="font-medium text-destructive">
                  {errorCount}
                </span>
              </span>
            )}
            <span className="text-muted-foreground">
              Total:{" "}
              <span className="font-medium text-foreground">
                {items.length}
              </span>
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
