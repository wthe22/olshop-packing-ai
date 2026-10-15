"""Check the PDFs the Rust spike wrote: font embedding and text readback (pypdf).

Reads only out/fonts.pdf and out/packing-list.pdf (both git-ignored, made-up content).
Prints byte sizes, the fonts with their subtype and whether a FontFile entry is present,
and how many expected strings the extracted text still contains.
"""

from __future__ import annotations

from pathlib import Path

from pypdf import PdfReader

OUT = Path(__file__).resolve().parent / "out"


def fonts_of(reader: PdfReader) -> dict[str, tuple[str, str]]:
    seen: dict[str, tuple[str, str]] = {}
    for page in reader.pages:
        res = page.get("/Resources")
        if res is None:
            continue
        res = res.get_object()
        fonts = res.get("/Font")
        if fonts is None:
            continue
        fonts = fonts.get_object()
        for key in fonts:
            obj = fonts[key].get_object()
            base = str(obj.get("/BaseFont"))
            subtype = str(obj.get("/Subtype"))
            embedded = "no"
            desc = obj.get("/FontDescriptor")
            if desc is not None:
                d = desc.get_object()
                for fk in ("/FontFile", "/FontFile2", "/FontFile3"):
                    if fk in d:
                        embedded = fk
            seen[base] = (subtype, embedded)
    return seen


def report(name: str, checks: list[str]) -> None:
    path = OUT / name
    reader = PdfReader(str(path))
    text = "".join((page.extract_text() or "") for page in reader.pages)
    print(f"{name}: bytes={path.stat().st_size} pages={len(reader.pages)} text_chars={len(text)}")
    for base, (subtype, embedded) in sorted(fonts_of(reader).items()):
        subset = "subset" if base.startswith("/") and "+" in base else "whole"
        print(f"   font {base}  subtype={subtype}  embedded={embedded}  ({subset})")
    found = sum(1 for c in checks if c in text)
    print(f"   readback: {found}/{len(checks)} expected strings found")
    missing = [c for c in checks if c not in text]
    if missing:
        print(f"   missing: {missing}")


def main() -> None:
    report(
        "fonts.pdf",
        [
            "Contoh dokumen uji",
            "Arial biasa",
            "Arial tebal",
            "Baris 1",
            "Baris 6",
            "JY0000001200",
            "JY0000001205",
            "Aa Bb Cc 0123456789",
            "contoh teks kecil",
        ],
    )
    report(
        "packing-list.pdf",
        [
            "Packing list",
            "contoh halaman 1",
            "contoh halaman 5",
            "Product Name",
            "Seller SKU",
            "Tracking ID",
            "Produk Contoh Alfa",
            "Produk Contoh Beta Panjang Sekali",
            "SELL-CONT-001",
            "JY0000001001",
            "JY0000001150",
        ],
    )


if __name__ == "__main__":
    main()
