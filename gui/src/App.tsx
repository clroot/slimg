import { useState } from "react";
import { Sidebar, type Feature } from "@/components/Sidebar";
import { DropZone } from "@/components/DropZone";
import { ImagePreview } from "@/components/ImagePreview";
import { OptionsPanel } from "@/components/OptionsPanel";
import { ProcessResultCard } from "@/components/ProcessResultCard";
import { BatchList } from "@/components/BatchList";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useImageProcess } from "@/hooks/useImageProcess";
import { useBatchProcess } from "@/hooks/useBatchProcess";
import { formatBytes } from "@/lib/format";
import { basename, capitalize } from "@/lib/path";
import { api, type ImageInfo, type ProcessOptions } from "@/lib/tauri";

interface LoadedFile {
  path: string;
  info: ImageInfo;
}

function App() {
  const [activeFeature, setActiveFeature] = useState<Feature>("convert");
  const [showSettings, setShowSettings] = useState(false);
  const [files, setFiles] = useState<LoadedFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [options, setOptions] = useState<Partial<ProcessOptions>>({});
  const {
    processing,
    result,
    resultInfo,
    error: processError,
    processImage,
    reset: resetResult,
  } = useImageProcess();
  const {
    batchItems,
    isProcessing: batchProcessing,
    progress: batchProgress,
    processBatch,
    reset: resetBatch,
  } = useBatchProcess();
  const [selectedBatchIndex, setSelectedBatchIndex] = useState<
    number | undefined
  >(undefined);

  const isBatchMode = files.length > 1;
  const hasBatchResult = batchItems.length > 0;

  const buildOptions = (): ProcessOptions => ({
    quality: 80,
    ...options,
    operation: activeFeature,
    overwrite: false,
  });

  const handleProcess = async () => {
    if (files.length === 0) return;

    if (isBatchMode) {
      setSelectedBatchIndex(undefined);
      await processBatch(
        files.map((f) => f.path),
        buildOptions()
      );
    } else {
      await processImage(files[0].path, buildOptions());
    }
  };

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
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to load images:", err);
    } finally {
      setLoading(false);
    }
  };

  const handleClearFiles = () => {
    setFiles([]);
    setError(null);
    resetResult();
    resetBatch();
    setSelectedBatchIndex(undefined);
  };

  const handleReset = () => {
    if (isBatchMode) {
      resetBatch();
      setSelectedBatchIndex(undefined);
    } else {
      resetResult();
    }
  };

  const isProcessing = isBatchMode ? batchProcessing : processing;
  const hasResult = isBatchMode ? hasBatchResult : result !== null;
  const currentError = isBatchMode ? null : processError;

  return (
    <div className="flex h-screen bg-background text-foreground">
      <Sidebar
        active={activeFeature}
        onSelect={(feature) => {
          setActiveFeature(feature);
          setShowSettings(false);
          setOptions({});
          resetResult();
          resetBatch();
          setSelectedBatchIndex(undefined);
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
                  {hasResult
                    ? "Processing complete"
                    : files.length > 0
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
            ) : hasResult ? (
              <ResultView
                isBatchMode={isBatchMode}
                files={files}
                result={result}
                resultInfo={resultInfo}
                batchItems={batchItems}
                batchProgress={batchProgress}
                batchProcessing={batchProcessing}
                selectedBatchIndex={selectedBatchIndex}
                onBatchItemClick={setSelectedBatchIndex}
                onReset={handleReset}
                onClear={handleClearFiles}
              />
            ) : (
              <FileListWithOptions
                files={files}
                activeFeature={activeFeature}
                isProcessing={isProcessing}
                isBatchMode={isBatchMode}
                processError={currentError}
                onOptionsChange={setOptions}
                onProcess={handleProcess}
              />
            )}
          </div>
        )}
      </main>
    </div>
  );
}

function ResultView({
  isBatchMode,
  files,
  result,
  resultInfo,
  batchItems,
  batchProgress,
  batchProcessing,
  selectedBatchIndex,
  onBatchItemClick,
  onReset,
  onClear,
}: {
  isBatchMode: boolean;
  files: LoadedFile[];
  result: ReturnType<typeof useImageProcess>["result"];
  resultInfo: ReturnType<typeof useImageProcess>["resultInfo"];
  batchItems: ReturnType<typeof useBatchProcess>["batchItems"];
  batchProgress: number;
  batchProcessing: boolean;
  selectedBatchIndex?: number;
  onBatchItemClick: (index: number) => void;
  onReset: () => void;
  onClear: () => void;
}) {
  if (isBatchMode) {
    const selectedItem =
      selectedBatchIndex !== undefined
        ? batchItems[selectedBatchIndex]
        : undefined;
    const selectedFile =
      selectedBatchIndex !== undefined
        ? files[selectedBatchIndex]
        : undefined;

    return (
      <div className="space-y-6">
        <BatchList
          items={batchItems}
          progress={batchProgress}
          onItemClick={onBatchItemClick}
          selectedIndex={selectedBatchIndex}
        />

        {selectedItem?.status === "completed" &&
          selectedItem.result &&
          selectedFile && (
            <div className="space-y-4">
              <h2 className="text-sm font-medium text-muted-foreground">
                File detail
              </h2>
              <ImagePreview
                original={selectedFile}
                result={selectedItem.result}
              />
              <ProcessResultCard result={selectedItem.result} />
            </div>
          )}

        {!batchProcessing && (
          <div className="flex gap-4">
            <Button
              variant="outline"
              onClick={onReset}
              className="flex-1"
              size="lg"
            >
              Process Again
            </Button>
            <Button
              variant="outline"
              onClick={onClear}
              className="flex-1"
              size="lg"
            >
              Clear
            </Button>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <ImagePreview
        original={files[0]}
        result={result!}
        resultInfo={resultInfo ?? undefined}
      />
      <ProcessResultCard result={result!} />
      <div className="flex gap-4">
        <Button
          variant="outline"
          onClick={onReset}
          className="flex-1"
          size="lg"
        >
          Process Again
        </Button>
        <Button
          variant="outline"
          onClick={onClear}
          className="flex-1"
          size="lg"
        >
          Clear
        </Button>
      </div>
    </div>
  );
}

function FileListWithOptions({
  files,
  activeFeature,
  isProcessing,
  isBatchMode,
  processError,
  onOptionsChange,
  onProcess,
}: {
  files: LoadedFile[];
  activeFeature: Feature;
  isProcessing: boolean;
  isBatchMode: boolean;
  processError: string | null;
  onOptionsChange: (options: Partial<ProcessOptions>) => void;
  onProcess: () => void;
}) {
  return (
    <>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {files.map((file) => (
          <div
            key={file.path}
            className="overflow-hidden rounded-xl border bg-card"
          >
            <div className="flex aspect-video items-center justify-center bg-muted">
              <img
                src={`data:image/png;base64,${file.info.thumbnail_base64}`}
                alt={basename(file.path)}
                className="max-h-full max-w-full object-contain"
              />
            </div>
            <div className="p-4">
              <p className="truncate text-sm font-medium">
                {basename(file.path)}
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
      <Separator className="my-6" />
      <div className="max-w-md space-y-6">
        <OptionsPanel
          feature={activeFeature}
          imageInfo={files[0]?.info}
          onChange={onOptionsChange}
        />
        <Button
          onClick={onProcess}
          disabled={isProcessing}
          className="w-full"
          size="lg"
        >
          {isProcessing
            ? "Processing..."
            : isBatchMode
              ? `${capitalize(activeFeature)} All (${files.length} files)`
              : `${capitalize(activeFeature)} Image`}
        </Button>

        {processError && (
          <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4">
            <p className="text-sm font-medium text-destructive">Error</p>
            <p className="mt-1 text-sm text-destructive/90">{processError}</p>
          </div>
        )}
      </div>
    </>
  );
}

export default App;
