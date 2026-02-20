import { useState, useCallback, useEffect } from "react";
import type { ProcessOptions } from "@/lib/tauri";
import { FormatSelect } from "./FormatSelect";
import { QualitySlider } from "./QualitySlider";

const DEFAULT_FORMAT = "jpeg";
const DEFAULT_QUALITY = 80;

interface ConvertOptionsProps {
  onChange: (options: Partial<ProcessOptions>) => void;
}

export function ConvertOptions({ onChange }: ConvertOptionsProps) {
  const [format, setFormat] = useState(DEFAULT_FORMAT);
  const [quality, setQuality] = useState(DEFAULT_QUALITY);

  const emitChange = useCallback(
    (fmt: string, q: number) => {
      onChange({ operation: "convert", format: fmt, quality: q });
    },
    [onChange]
  );

  useEffect(() => {
    emitChange(format, quality);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- emit initial values on mount only
  }, []);

  const handleFormatChange = (value: string) => {
    setFormat(value);
    emitChange(value, quality);
  };

  const handleQualityChange = (value: number) => {
    setQuality(value);
    emitChange(format, value);
  };

  return (
    <div className="space-y-4">
      <FormatSelect value={format} onChange={handleFormatChange} />
      <QualitySlider value={quality} onChange={handleQualityChange} />
    </div>
  );
}
