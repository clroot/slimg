import type { Feature } from "@/components/Sidebar";
import type { ProcessOptions, ImageInfo } from "@/lib/tauri";
import { ConvertOptions } from "@/components/options/ConvertOptions";
import { OptimizeOptions } from "@/components/options/OptimizeOptions";
import { ResizeOptions } from "@/components/options/ResizeOptions";
import { CropOptions } from "@/components/options/CropOptions";
import { ExtendOptions } from "@/components/options/ExtendOptions";

interface OptionsPanelProps {
  feature: Feature;
  imageInfo?: ImageInfo;
  onChange: (options: Partial<ProcessOptions>) => void;
}

export function OptionsPanel({
  feature,
  imageInfo,
  onChange,
}: OptionsPanelProps) {
  switch (feature) {
    case "convert":
      return <ConvertOptions onChange={onChange} />;
    case "optimize":
      return <OptimizeOptions onChange={onChange} />;
    case "resize":
      return <ResizeOptions onChange={onChange} imageInfo={imageInfo} />;
    case "crop":
      return <CropOptions onChange={onChange} />;
    case "extend":
      return <ExtendOptions onChange={onChange} />;
  }
}
