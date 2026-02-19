import { useState, useCallback, useEffect } from "react";
import type { ProcessOptions } from "@/lib/tauri";
import { QualitySlider } from "./QualitySlider";

const DEFAULT_QUALITY = 80;

interface OptimizeOptionsProps {
  onChange: (options: Partial<ProcessOptions>) => void;
}

export function OptimizeOptions({ onChange }: OptimizeOptionsProps) {
  const [quality, setQuality] = useState(DEFAULT_QUALITY);

  const emitChange = useCallback(
    (q: number) => {
      onChange({ operation: "optimize", quality: q });
    },
    [onChange]
  );

  useEffect(() => {
    emitChange(quality);
  }, []);

  const handleQualityChange = (value: number) => {
    setQuality(value);
    emitChange(value);
  };

  return (
    <div className="space-y-4">
      <QualitySlider value={quality} onChange={handleQualityChange} />
    </div>
  );
}
