// A size on disk as the Archive page shows it: `KB`, `MB` or `GB`, one decimal under 10
// ("0.1 MB", "84 MB", "1.2 GB"). A size from a tenth of a megabyte is in megabytes, from one
// gigabyte in gigabytes.

const KB = 1024;
const MB = KB * 1024;
const GB = MB * 1024;

function number(value: number): string {
  return value < 9.95 ? value.toFixed(1) : String(Math.round(value));
}

export function formatSize(bytes: number): string {
  if (bytes >= GB) return `${number(bytes / GB)} GB`;
  if (bytes >= MB / 10) return `${number(bytes / MB)} MB`;
  return `${number(bytes / KB)} KB`;
}
