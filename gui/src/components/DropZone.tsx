import { useEffect, useState, useCallback } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";
import { Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

const SUPPORTED_EXTENSIONS = [
  "jpg",
  "jpeg",
  "png",
  "webp",
  "avif",
  "jxl",
  "qoi",
];

function isSupportedImage(path: string): boolean {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  return SUPPORTED_EXTENSIONS.includes(ext);
}

interface DropZoneProps {
  onFilesSelected: (paths: string[]) => void;
  disabled?: boolean;
}

export function DropZone({ onFilesSelected, disabled = false }: DropZoneProps) {
  const [isDragOver, setIsDragOver] = useState(false);

  useEffect(() => {
    const webview = getCurrentWebviewWindow();

    const unlistenPromise = webview.onDragDropEvent((event) => {
      if (disabled) return;

      const { type } = event.payload;

      if (type === "enter" || type === "over") {
        setIsDragOver(true);
      } else if (type === "drop") {
        setIsDragOver(false);
        const supported = event.payload.paths.filter(isSupportedImage);
        if (supported.length > 0) {
          onFilesSelected(supported);
        }
      } else if (type === "leave") {
        setIsDragOver(false);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [onFilesSelected, disabled]);

  const handleBrowse = useCallback(async () => {
    if (disabled) return;

    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "Images",
          extensions: SUPPORTED_EXTENSIONS,
        },
      ],
    });

    if (selected && selected.length > 0) {
      onFilesSelected(selected);
    }
  }, [onFilesSelected, disabled]);

  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-6 rounded-xl border-2 border-dashed p-12 transition-colors",
        isDragOver
          ? "border-primary bg-primary/5"
          : "border-muted-foreground/25 bg-muted/30",
        disabled && "pointer-events-none opacity-50"
      )}
    >
      <div
        className={cn(
          "flex h-16 w-16 items-center justify-center rounded-full transition-colors",
          isDragOver ? "bg-primary/10 text-primary" : "bg-muted text-muted-foreground"
        )}
      >
        <Upload className="h-8 w-8" />
      </div>

      <div className="text-center">
        <p className="text-lg font-medium">
          {isDragOver ? "Drop images here" : "Drop images here"}
        </p>
        <p className="mt-1 text-sm text-muted-foreground">
          Supports JPG, PNG, WebP, AVIF, JXL, QOI
        </p>
      </div>

      <Button variant="outline" size="lg" onClick={handleBrowse} disabled={disabled}>
        Browse Files
      </Button>
    </div>
  );
}
