export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function calcSavingsPercent(
  originalSize: number,
  newSize: number
): number {
  return originalSize > 0 ? (1 - newSize / originalSize) * 100 : 0;
}
