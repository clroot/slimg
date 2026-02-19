import { useState } from "react";
import { Sidebar, type Feature } from "@/components/Sidebar";
import { DropZone } from "@/components/DropZone";
import { api, type ImageInfo } from "@/lib/tauri";

interface LoadedFile {
  path: string;
  info: ImageInfo;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function App() {
  const [activeFeature, setActiveFeature] = useState<Feature>("convert");
  const [showSettings, setShowSettings] = useState(false);
  const [files, setFiles] = useState<LoadedFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFilesSelected = async (paths: string[]) => {
    setLoading(true);
    setError(null);
    try {
      const loaded = await Promise.all(
        paths.map(async (path) => ({
          path,
          info: await api.loadImage(path),
        }))
      );
      setFiles(loaded);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to load images:", err);
    } finally {
      setLoading(false);
    }
  };

  const handleClearFiles = () => {
    setFiles([]);
    setError(null);
  };

  return (
    <div className="flex h-screen bg-background text-foreground">
      <Sidebar
        active={activeFeature}
        onSelect={(feature) => {
          setActiveFeature(feature);
          setShowSettings(false);
        }}
        onSettingsClick={() => setShowSettings(true)}
      />
      <main className="flex-1 overflow-auto p-6">
        {showSettings ? (
          <div>
            <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
            <p className="mt-1 text-muted-foreground">
              Application settings (TODO)
            </p>
          </div>
        ) : (
          <div className="flex h-full flex-col">
            <div className="mb-6 flex items-center justify-between">
              <div>
                <h1 className="text-2xl font-bold capitalize tracking-tight">
                  {activeFeature}
                </h1>
                <p className="mt-1 text-muted-foreground">
                  {files.length > 0
                    ? `${files.length} image${files.length > 1 ? "s" : ""} loaded`
                    : `Select images to ${activeFeature}`}
                </p>
              </div>
              {files.length > 0 && (
                <button
                  onClick={handleClearFiles}
                  className="text-sm text-muted-foreground transition-colors hover:text-foreground"
                >
                  Clear all
                </button>
              )}
            </div>

            {files.length === 0 ? (
              <div className="flex flex-1 items-center justify-center">
                <div className="w-full max-w-lg">
                  <DropZone
                    onFilesSelected={handleFilesSelected}
                    disabled={loading}
                  />
                  {loading && (
                    <p className="mt-4 text-center text-sm text-muted-foreground">
                      Loading images...
                    </p>
                  )}
                  {error && (
                    <p className="mt-4 text-center text-sm text-destructive">
                      {error}
                    </p>
                  )}
                </div>
              </div>
            ) : (
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {files.map((file) => (
                  <div
                    key={file.path}
                    className="overflow-hidden rounded-xl border bg-card"
                  >
                    <div className="flex aspect-video items-center justify-center bg-muted">
                      <img
                        src={`data:image/png;base64,${file.info.thumbnail_base64}`}
                        alt={file.path.split("/").pop() ?? "image"}
                        className="max-h-full max-w-full object-contain"
                      />
                    </div>
                    <div className="p-4">
                      <p className="truncate text-sm font-medium">
                        {file.path.split("/").pop()}
                      </p>
                      <p className="mt-1 text-xs text-muted-foreground">
                        {file.info.width} x {file.info.height} &middot;{" "}
                        {file.info.format.toUpperCase()} &middot;{" "}
                        {formatBytes(file.info.size_bytes)}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
