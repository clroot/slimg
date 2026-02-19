import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";

interface QualitySliderProps {
  value: number;
  onChange: (value: number) => void;
}

export function QualitySlider({ value, onChange }: QualitySliderProps) {
  return (
    <div className="space-y-2">
      <Label>Quality: {value}</Label>
      <Slider
        value={[value]}
        onValueChange={([v]) => onChange(v)}
        min={0}
        max={100}
        step={1}
      />
    </div>
  );
}
