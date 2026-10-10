#!/usr/bin/env bash
# Build the portable program folder (07 › *Program folder*, task 2.13): one folder with
# Packing.exe and pdfium.dll, no installer. The owner copies the folder to e.g. D:\Packing\
# and starts Packing.exe from a shortcut; the data (categories.toml, settings.json, labels/)
# lives in the same folder.
#
# Steps:
#   1. build the release binary without bundling (npm run tauri build -- --no-bundle)
#   2. copy the binary (renamed Packing.exe) and pdfium.dll into
#      pc/target/portable/Packing/
#
# Run from anywhere:  bash pc/tools/make-portable.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
pc="$(cd "$here/.." && pwd)"
app="$pc/app"
vendor="$pc/vendor"
release="$pc/target/release"
portable="$pc/target/portable"
out="$portable/Packing"

# pdfium.dll is git-ignored; fetch it once with get-pdfium.sh (01-setup).
if [ ! -f "$vendor/pdfium.dll" ]; then
  echo "error: $vendor/pdfium.dll is missing." >&2
  echo "Run 'bash pc/tools/get-pdfium.sh' first: it downloads the pinned PDFium (chromium/7881)." >&2
  exit 1
fi

echo "==> Building the release program (npm run tauri build -- --no-bundle)"
( cd "$app" && npm run tauri build -- --no-bundle )

# Tauri names the built binary after the productName in tauri.conf.json ("Packing"); if that
# rename did not happen it is the Cargo package name (packing-app). Accept either.
product="$(sed -n 's/.*"productName"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$app/src-tauri/tauri.conf.json" | head -n1)"
exe=""
for name in "$product" "packing-app"; do
  if [ -n "$name" ] && [ -f "$release/$name.exe" ]; then
    exe="$release/$name.exe"
    break
  fi
done
if [ -z "$exe" ]; then
  echo "error: no release .exe found in $release (looked for ${product:-?}.exe and packing-app.exe)" >&2
  exit 1
fi

echo "==> Assembling the program folder at $out"
rm -rf "$portable"
mkdir -p "$out"
cp "$exe" "$out/Packing.exe"
cp "$vendor/pdfium.dll" "$out/pdfium.dll"

echo "==> Done"
ls -la "$out"
echo "Program folder: $out"
