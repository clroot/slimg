import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Slider } from "@/components/ui/slider";
import type { AppSettings } from "@/hooks/useSettings";

interface SettingsProps {
  settings: AppSettings;
  onUpdate: (patch: Partial<AppSettings>) => void;
  onReset: () => void;
}

export function Settings({ settings, onUpdate, onReset }: SettingsProps) {
  const handleSelectOutputDir = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      onUpdate({ outputDir: selected });
    }
  };

  return (
    <div className="flex h-full flex-col">
      <div className="mb-6">
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
        <p className="mt-1 text-muted-foreground">
          Configure default processing options
        </p>
      </div>

      <div className="max-w-lg space-y-8">
        <section className="space-y-4">
          <h2 className="text-sm font-semibold uppercase tracking-widest text-muted-foreground">
            Output
          </h2>

          <div className="space-y-2">
            <Label htmlFor="output-dir">Default output directory</Label>
            <div className="flex gap-2">
              <Input
                id="output-dir"
                value={settings.outputDir}
                onChange={(e) => onUpdate({ outputDir: e.target.value })}
                placeholder="Same as input file"
                className="flex-1"
                readOnly
              />
              <Button
                variant="outline"
                size="icon"
                onClick={handleSelectOutputDir}
                className="shrink-0"
              >
                <FolderOpen className="h-4 w-4" />
                <span className="sr-only">Browse folder</span>
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              Leave empty to save output next to the original file
            </p>
          </div>

          <div className="flex items-center gap-3">
            <Checkbox
              id="overwrite"
              checked={settings.overwrite}
              onCheckedChange={(checked) =>
                onUpdate({ overwrite: checked === true })
              }
            />
            <Label htmlFor="overwrite" className="cursor-pointer">
              Overwrite original files
            </Label>
          </div>
        </section>

        <Separator />

        <section className="space-y-4">
          <h2 className="text-sm font-semibold uppercase tracking-widest text-muted-foreground">
            Quality
          </h2>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <Label htmlFor="quality">Default quality</Label>
              <span className="text-sm font-medium tabular-nums">
                {settings.defaultQuality}%
              </span>
            </div>
            <Slider
              id="quality"
              value={[settings.defaultQuality]}
              onValueChange={([value]) =>
                onUpdate({ defaultQuality: value })
              }
              min={1}
              max={100}
              step={1}
            />
            <p className="text-xs text-muted-foreground">
              Used when quality is not explicitly set per operation
            </p>
          </div>
        </section>

        <Separator />

        <Button variant="outline" onClick={onReset} className="gap-2">
          <RotateCcw className="h-4 w-4" />
          Reset to defaults
        </Button>
      </div>
    </div>
  );
}
