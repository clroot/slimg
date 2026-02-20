import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const FORMATS = [
  { value: "jpeg", label: "JPEG" },
  { value: "png", label: "PNG" },
  { value: "webp", label: "WebP" },
  { value: "avif", label: "AVIF" },
  { value: "qoi", label: "QOI" },
  { value: "jxl", label: "JXL" },
] as const;

interface FormatSelectProps {
  value: string;
  onChange: (value: string) => void;
  includeSameAsInput?: boolean;
  label?: string;
}

export function FormatSelect({
  value,
  onChange,
  includeSameAsInput = false,
  label = "Output Format",
}: FormatSelectProps) {
  return (
    <div className="space-y-2">
      <Label>{label}</Label>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {includeSameAsInput && (
            <SelectItem value="same">Same as input</SelectItem>
          )}
          {FORMATS.map((fmt) => (
            <SelectItem key={fmt.value} value={fmt.value}>
              {fmt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
