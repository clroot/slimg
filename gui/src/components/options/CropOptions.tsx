import { useState, useCallback, useEffect } from "react";
import type { ProcessOptions } from "@/lib/tauri";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { FormatSelect } from "./FormatSelect";
import { QualitySlider } from "./QualitySlider";

type CropMode = "region" | "aspect";

interface CropOptionsProps {
  defaultQuality: number;
  onChange: (options: Partial<ProcessOptions>) => void;
}

export function CropOptions({ defaultQuality, onChange }: CropOptionsProps) {
  const [cropMode, setCropMode] = useState<CropMode>("region");
  const [x, setX] = useState<number | undefined>(undefined);
  const [y, setY] = useState<number | undefined>(undefined);
  const [width, setWidth] = useState<number | undefined>(undefined);
  const [height, setHeight] = useState<number | undefined>(undefined);
  const [format, setFormat] = useState("same");
  const [quality, setQuality] = useState(defaultQuality);

  const emitChange = useCallback(
    (state: {
      cropMode: CropMode;
      x?: number;
      y?: number;
      width?: number;
      height?: number;
      format: string;
      quality: number;
    }) => {
      const opts: Partial<ProcessOptions> = {
        operation: "crop",
        crop_mode: state.cropMode,
        quality: state.quality,
      };
      if (state.cropMode === "region") {
        if (state.x !== undefined) opts.x = state.x;
        if (state.y !== undefined) opts.y = state.y;
      }
      if (state.width) opts.width = state.width;
      if (state.height) opts.height = state.height;
      if (state.format !== "same") opts.format = state.format;
      onChange(opts);
    },
    [onChange]
  );

  useEffect(() => {
    emitChange({ cropMode, x, y, width, height, format, quality });
    // eslint-disable-next-line react-hooks/exhaustive-deps -- emit initial values on mount only
  }, []);

  const handleCropModeChange = (value: string) => {
    const mode = value as CropMode;
    setCropMode(mode);
    emitChange({ cropMode: mode, x, y, width, height, format, quality });
  };

  const parseNum = (value: string) =>
    value ? parseInt(value, 10) : undefined;

  const handleFieldChange = (
    field: "x" | "y" | "width" | "height",
    value: string
  ) => {
    const num = parseNum(value);
    const next = { cropMode, x, y, width, height, format, quality };
    next[field] = num;

    if (field === "x") setX(num);
    if (field === "y") setY(num);
    if (field === "width") setWidth(num);
    if (field === "height") setHeight(num);

    emitChange(next);
  };

  const handleFormatChange = (value: string) => {
    setFormat(value);
    emitChange({ cropMode, x, y, width, height, format: value, quality });
  };

  const handleQualityChange = (value: number) => {
    setQuality(value);
    emitChange({ cropMode, x, y, width, height, format, quality: value });
  };

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>Mode</Label>
        <RadioGroup
          value={cropMode}
          onValueChange={handleCropModeChange}
          className="grid grid-cols-2 gap-2"
        >
          <label className="flex cursor-pointer items-center gap-2 rounded-lg border px-3 py-2.5 transition-colors hover:bg-accent">
            <RadioGroupItem value="region" />
            <span className="text-sm">Region</span>
          </label>
          <label className="flex cursor-pointer items-center gap-2 rounded-lg border px-3 py-2.5 transition-colors hover:bg-accent">
            <RadioGroupItem value="aspect" />
            <span className="text-sm">Aspect Ratio</span>
          </label>
        </RadioGroup>
      </div>

      {cropMode === "region" && (
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label>X</Label>
            <Input
              type="number"
              min={0}
              value={x ?? ""}
              onChange={(e) => handleFieldChange("x", e.target.value)}
              placeholder="0"
            />
          </div>
          <div className="space-y-2">
            <Label>Y</Label>
            <Input
              type="number"
              min={0}
              value={y ?? ""}
              onChange={(e) => handleFieldChange("y", e.target.value)}
              placeholder="0"
            />
          </div>
        </div>
      )}

      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-2">
          <Label>Width</Label>
          <Input
            type="number"
            min={1}
            value={width ?? ""}
            onChange={(e) => handleFieldChange("width", e.target.value)}
            placeholder="Width"
          />
        </div>
        <div className="space-y-2">
          <Label>Height</Label>
          <Input
            type="number"
            min={1}
            value={height ?? ""}
            onChange={(e) => handleFieldChange("height", e.target.value)}
            placeholder="Height"
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
