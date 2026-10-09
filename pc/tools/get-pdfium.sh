#!/usr/bin/env bash
# Download a pinned prebuilt Pdfium (bblanchon/pdfium-binaries) for Windows x64 and
# extract pdfium.dll into pc/vendor/ (git-ignored).
#
# The tag is pinned to match the pdfium-render version in pc/spike/Cargo.toml:
#   pdfium-render 0.9.4  -- feature `pdfium_latest` == `pdfium_7881`
#   -> bblanchon release tag `chromium/7881`
#
# Re-run to re-download; the file is verified against SHA256 below.
set -euo pipefail

PDFIUM_TAG="chromium/7881"
ASSET="pdfium-win-x64.tgz"
# sha256 of pdfium-win-x64.tgz for tag chromium/7881.
PDFIUM_SHA256="73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08"

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
vendor="$here/../vendor"
mkdir -p "$vendor"

url="https://github.com/bblanchon/pdfium-binaries/releases/download/${PDFIUM_TAG//\//%2F}/${ASSET}"
tmp="$vendor/.tmp"
rm -rf "$tmp"; mkdir -p "$tmp"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $url"
# curl fails to write from release-assets.githubusercontent.com on this host
# (schannel write error, curl 23); fall back to Python's urllib.
if ! curl -fsSL "$url" -o "$tmp/$ASSET" 2>/dev/null; then
  echo "curl failed; using python"
  PY=""
  for cand in "$here/../../.venv/Scripts/python.exe" python3 python; do
    if command -v "$cand" >/dev/null 2>&1; then PY="$cand"; break; fi
  done
  [ -n "$PY" ] || { echo "no python found for download" >&2; exit 1; }
  dest_win="$(cygpath -w "$tmp/$ASSET" 2>/dev/null || printf '%s' "$tmp/$ASSET")"
  "$PY" - "$url" "$dest_win" <<'PYEOF'
import sys, urllib.request
url, dest = sys.argv[1], sys.argv[2]
with urllib.request.urlopen(url, timeout=120) as r, open(dest, "wb") as f:
    f.write(r.read())
PYEOF
fi

if [ "$PDFIUM_SHA256" != "__FILL_AFTER_FIRST_RUN__" ]; then
  actual="$(sha256sum "$tmp/$ASSET" | cut -d' ' -f1)"
  if [ "$actual" != "$PDFIUM_SHA256" ]; then
    echo "SHA256 mismatch: expected $PDFIUM_SHA256, got $actual" >&2
    exit 1
  fi
  echo "SHA256 ok: $actual"
else
  echo "SHA256 (pin this in the script): $(sha256sum "$tmp/$ASSET" | cut -d' ' -f1)"
fi

tar -xzf "$tmp/$ASSET" -C "$tmp"
dll="$(find "$tmp" -name 'pdfium.dll' | head -n1)"
if [ -z "$dll" ]; then
  echo "pdfium.dll not found in archive" >&2
  exit 1
fi
cp "$dll" "$vendor/pdfium.dll"
echo "Wrote $vendor/pdfium.dll ($(stat -c%s "$vendor/pdfium.dll") bytes)"
