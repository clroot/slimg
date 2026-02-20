import { useState, useCallback, useEffect } from "react";
import type { ProcessOptions } from "@/lib/tauri";
import { QualitySlider } from "./QualitySlider";

interface OptimizeOptionsProps {
  defaultQuality: number;
  onChange: (options: Partial<ProcessOptions>) => void;
}

export function OptimizeOptions({ defaultQuality, onChange }: OptimizeOptionsProps) {
  const [quality, setQuality] = useState(defaultQuality);

  const emitChange = useCallback(
    (q: number) => {
      onChange({ operation: "optimize", quality: q });
    },
    [onChange]
  );

  useEffect(() => {
    emitChange(quality);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- emit initial values on mount only
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
