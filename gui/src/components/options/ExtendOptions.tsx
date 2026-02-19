import { useState, useCallback, useEffect } from "react";
import type { ProcessOptions } from "@/lib/tauri";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { FormatSelect } from "./FormatSelect";
import { QualitySlider } from "./QualitySlider";

const DEFAULT_QUALITY = 80;
const DEFAULT_FILL_COLOR = "#FFFFFF";

interface ExtendOptionsProps {
  onChange: (options: Partial<ProcessOptions>) => void;
}

export function ExtendOptions({ onChange }: ExtendOptionsProps) {
  const [width, setWidth] = useState<number | undefined>(undefined);
  const [height, setHeight] = useState<number | undefined>(undefined);
  const [fillColor, setFillColor] = useState(DEFAULT_FILL_COLOR);
  const [format, setFormat] = useState("same");
  const [quality, setQuality] = useState(DEFAULT_QUALITY);

  const emitChange = useCallback(
    (state: {
      width?: number;
      height?: number;
      fillColor: string;
      format: string;
      quality: number;
    }) => {
      const opts: Partial<ProcessOptions> = {
        operation: "extend",
        fill_color: state.fillColor,
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
    emitChange({ width, height, fillColor, format, quality });
  }, []);

  const handleWidthChange = (value: string) => {
    const w = value ? parseInt(value, 10) : undefined;
    setWidth(w);
    emitChange({ width: w, height, fillColor, format, quality });
  };

  const handleHeightChange = (value: string) => {
    const h = value ? parseInt(value, 10) : undefined;
    setHeight(h);
    emitChange({ width, height: h, fillColor, format, quality });
  };

  const handleFillColorChange = (value: string) => {
    setFillColor(value);
    emitChange({ width, height, fillColor: value, format, quality });
  };

  const handleFormatChange = (value: string) => {
    setFormat(value);
    emitChange({ width, height, fillColor, format: value, quality });
  };

  const handleQualityChange = (value: number) => {
    setQuality(value);
    emitChange({ width, height, fillColor, format, quality: value });
  };

  const isValidHex = /^#[0-9A-Fa-f]{6}$/.test(fillColor);

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>Target Aspect Ratio</Label>
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">Width</Label>
            <Input
              type="number"
              min={1}
              value={width ?? ""}
              onChange={(e) => handleWidthChange(e.target.value)}
              placeholder="Width"
            />
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">Height</Label>
            <Input
              type="number"
              min={1}
              value={height ?? ""}
              onChange={(e) => handleHeightChange(e.target.value)}
              placeholder="Height"
            />
          </div>
        </div>
      </div>

      <div className="space-y-2">
        <Label>Fill Color</Label>
        <div className="flex items-center gap-2">
          <div
            className="h-9 w-9 shrink-0 rounded-md border"
            style={{
              backgroundColor: isValidHex ? fillColor : "#FFFFFF",
            }}
          />
          <Input
            value={fillColor}
            onChange={(e) => handleFillColorChange(e.target.value)}
            placeholder="#FFFFFF"
            maxLength={7}
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
