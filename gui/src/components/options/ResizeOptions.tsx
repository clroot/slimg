import { useState, useCallback, useEffect } from "react";
import type { ProcessOptions, ImageInfo } from "@/lib/tauri";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { FormatSelect } from "./FormatSelect";
import { QualitySlider } from "./QualitySlider";

type ResizeMode = "width" | "height" | "exact" | "fit";

const DEFAULT_QUALITY = 80;

const RESIZE_MODES: { value: ResizeMode; label: string }[] = [
  { value: "width", label: "By Width" },
  { value: "height", label: "By Height" },
  { value: "exact", label: "Exact" },
  { value: "fit", label: "Fit" },
];

interface ResizeOptionsProps {
  onChange: (options: Partial<ProcessOptions>) => void;
  imageInfo?: ImageInfo;
}

export function ResizeOptions({ onChange, imageInfo }: ResizeOptionsProps) {
  const [resizeMode, setResizeMode] = useState<ResizeMode>("width");
  const [width, setWidth] = useState<number | undefined>(undefined);
  const [height, setHeight] = useState<number | undefined>(undefined);
  const [format, setFormat] = useState("same");
  const [quality, setQuality] = useState(DEFAULT_QUALITY);

  const emitChange = useCallback(
    (state: {
      resizeMode: ResizeMode;
      width?: number;
      height?: number;
      format: string;
      quality: number;
    }) => {
      const opts: Partial<ProcessOptions> = {
        operation: "resize",
        resize_mode: state.resizeMode,
        quality: state.quality,
      };
      if (state.width) opts.width = state.width;
      if (state.height) opts.height = state.height;
      if (state.format !== "same") opts.format = state.format;
      onChange(opts);
    },
    [onChange]
  );

  useEffect(() => {
    emitChange({ resizeMode, width, height, format, quality });
    // eslint-disable-next-line react-hooks/exhaustive-deps -- emit initial values on mount only
  }, []);

  const handleResizeModeChange = (value: string) => {
    const mode = value as ResizeMode;
    setResizeMode(mode);
    emitChange({ resizeMode: mode, width, height, format, quality });
  };

  const handleWidthChange = (value: string) => {
    const w = value ? parseInt(value, 10) : undefined;
    setWidth(w);
    emitChange({ resizeMode, width: w, height, format, quality });
  };

  const handleHeightChange = (value: string) => {
    const h = value ? parseInt(value, 10) : undefined;
    setHeight(h);
    emitChange({ resizeMode, width, height: h, format, quality });
  };

  const handleFormatChange = (value: string) => {
    setFormat(value);
    emitChange({ resizeMode, width, height, format: value, quality });
  };

  const handleQualityChange = (value: number) => {
    setQuality(value);
    emitChange({ resizeMode, width, height, format, quality: value });
  };

  const isWidthDisabled = resizeMode === "height";
  const isHeightDisabled = resizeMode === "width";

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>Mode</Label>
        <RadioGroup
          value={resizeMode}
          onValueChange={handleResizeModeChange}
          className="grid grid-cols-2 gap-2"
        >
          {RESIZE_MODES.map((mode) => (
            <label
              key={mode.value}
              className="flex cursor-pointer items-center gap-2 rounded-lg border px-3 py-2.5 transition-colors hover:bg-accent"
            >
              <RadioGroupItem value={mode.value} />
              <span className="text-sm">{mode.label}</span>
            </label>
          ))}
        </RadioGroup>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-2">
          <Label>Width</Label>
          <Input
            type="number"
            min={1}
            value={width ?? ""}
            onChange={(e) => handleWidthChange(e.target.value)}
            placeholder={imageInfo ? String(imageInfo.width) : "Width"}
            disabled={isWidthDisabled}
          />
        </div>
        <div className="space-y-2">
          <Label>Height</Label>
          <Input
            type="number"
            min={1}
            value={height ?? ""}
            onChange={(e) => handleHeightChange(e.target.value)}
            placeholder={imageInfo ? String(imageInfo.height) : "Height"}
            disabled={isHeightDisabled}
          />
        </div>
      </div>

      <FormatSelect
        value={format}
        onChange={handleFormatChange}
        includeSameAsInput
      />
      <QualitySlider value={quality} onChange={handleQualityChange} />
    </div>
  );
}
