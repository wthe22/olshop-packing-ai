//! Throwaway spike: can `pdfium-render` 0.9.4 + prebuilt `pdfium.dll` replace the Python
//! pypdf/fpdf2 code for (1) reading label text with positions, (2) copying pages unchanged,
//! (3) writing the A4 packing list? See README.md for the measured results.
//!
//! Checks (each prints one line of counts/sizes/times; no real values are printed):
//!   a. Order ID found on every page (pass 1 / pass 2 / missed).
//!   b. Slip lines per order dumped to out/rust-orders.json (compared with Python by compare.py).
//!   c. 10 pages copied with copy_page_range_from_document, rendered and pixel-diffed.
//!   d. out/fonts.pdf (Arial/Consolas) and out/packing-list.pdf (5 A4 pages, made up).
//!   e. wall time to read the 3 files (text + Order ID + slip), Python runs the same work.

use std::cmp::Ordering;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use pdfium_render::prelude::*;
use serde_json::{json, Map, Value};

mod slip;
use slip::{find_order_id, slip, Run, Slip};

const DPI: f64 = 150.0;

fn main() {
    if let Err(err) = run() {
        eprintln!("spike error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("out");
    fs::create_dir_all(&out)?;
    let dll = manifest.join("../vendor/pdfium.dll");
    // SPIKE_LABELS=<dir> SPIKE_MATCH=<name part> runs on other files (e.g. testdata, labels-slip).
    let samples = std::env::var("SPIKE_LABELS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest.join("../../samples"));
    let wanted = std::env::var("SPIKE_MATCH").unwrap_or_else(|_| "Shipping label+Packing slip".into());

    let mut files: Vec<PathBuf> = fs::read_dir(&samples)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            name.contains(wanted.as_str()) && name.ends_with(".pdf")
        })
        .collect();
    files.sort();
    println!("files: {}", files.len());

    let pdfium = Pdfium::new(Pdfium::bind_to_library(&dll)?);
    let docs: Vec<PdfDocument> = files
        .iter()
        .map(|f| pdfium.load_pdf_from_file(f, None))
        .collect::<Result<_, _>>()?;

    // ---------------------------------------------------------------- read pass (a + b + e)
    let start = Instant::now();
    let mut pass1 = 0usize;
    let mut pass2 = 0usize;
    let mut missed = 0usize;
    let mut page_count = 0usize;

    // page key -> (order_id, optional slip), keeping the LabelSet.open order rules.
    let mut order_of_page: Vec<Vec<String>> = Vec::new();
    let mut page_slips: Vec<Vec<Option<Slip>>> = Vec::new();
    for (fi, doc) in docs.iter().enumerate() {
        let mut prev: Option<String> = None;
        let mut orders: Vec<String> = Vec::new();
        let mut slips: Vec<Option<Slip>> = Vec::new();
        for (pi, page) in doc.pages().iter().enumerate() {
            page_count += 1;
            let text = page
                .text()
                .map_err(|e| format!("file {fi} page {pi}: text(): {e}"))?;
            let all = text.all();
            let found = match find_order_id(&all) {
                Some((1, id)) => {
                    pass1 += 1;
                    id
                }
                Some((2, id)) => {
                    pass2 += 1;
                    id
                }
                _ => {
                    missed += 1;
                    String::new()
                }
            };
            let order_id = if found.is_empty() {
                match &prev {
                    Some(p) => p.clone(),
                    None => {
                        return Err(format!("file {} page has no Order ID", fi).into());
                    }
                }
            } else {
                found.clone()
            };
            prev = Some(order_id.clone());

            let slips_here = slip_for_page(&page, &text, &all, page.width().value as f64)
                .map_err(|e| format!("file {fi} page {pi}: slip: {e}"))?;
            orders.push(order_id);
            slips.push(slips_here.clone());
        }
        order_of_page.push(orders);
        page_slips.push(slips);
        let _ = fi;
    }

    let read_elapsed = start.elapsed();

    println!("pages: {}  order_id pass1: {} pass2: {} missed: {}", page_count, pass1, pass2, missed);

    // group by order id (first-seen order), mirroring LabelSet.by_order + orders()
    let mut order_pages: Vec<(String, Vec<(usize, usize)>)> = Vec::new();
    for (fi, orders) in order_of_page.iter().enumerate() {
        for (pi, oid) in orders.iter().enumerate() {
            match order_pages.iter_mut().find(|(id, _)| id == oid) {
                Some((_, v)) => v.push((fi, pi)),
                None => order_pages.push((oid.clone(), vec![(fi, pi)])),
            }
        }
    }

    let mut dump: Map<String, Value> = Map::new();
    for (oid, pages) in order_pages.iter() {
        let mut lines: Vec<Value> = Vec::new();
        let mut qty_total: Option<i64> = None;
        let mut customer_message = String::new();
        for (fi, pi) in pages {
            if let Some(sl) = &page_slips[*fi][*pi] {
                for line in &sl.lines {
                    lines.push(json!({
                        "name": line.name,
                        "variation": line.variation,
                        "seller_sku": line.seller_sku,
                        "quantity": line.quantity,
                    }));
                }
                if sl.qty_total.is_some() {
                    qty_total = sl.qty_total;
                }
                if !sl.customer_message.is_empty() {
                    customer_message = sl.customer_message.clone();
                }
            }
        }
        let page_refs: Vec<Value> = pages
            .iter()
            .map(|(fi, pi)| {
                json!({
                    "file": files[*fi].file_name().and_then(|s| s.to_str()).unwrap_or(""),
                    "index": pi,
                })
            })
            .collect();
        dump.insert(
            oid.clone(),
            json!({
                "lines": lines,
                "qty_total": qty_total,
                "customer_message": customer_message,
                "pages": page_refs,
            }),
        );
    }
    let rust_json = out.join("rust-orders.json");
    write_pretty(&rust_json, &Value::Object(dump.clone()))?;
    println!(
        "orders: {}  rust-orders.json: {} bytes",
        order_pages.len(),
        fs::metadata(&rust_json)?.len()
    );
    println!("read wall time: {:.3} s", read_elapsed.as_secs_f64());

    // ---------------------------------------------------------------- copy + render (c)
    check_copy(&pdfium, &docs, &files, &order_pages, &out)?;

    // ---------------------------------------------------------------- write PDFs (d)
    check_write(&pdfium, &out)?;

    Ok(())
}

/// Build the runs of a page and its slip, when the page carries a slip header.
fn slip_for_page(
    page: &PdfPage,
    text: &PdfPageText,
    all: &str,
    page_width: f64,
) -> Result<Option<Slip>, String> {
    let collapsed: String = all.chars().filter(|c| !c.is_whitespace()).collect();
    if !(all.contains("Product Name") || collapsed.contains("ProductName")) {
        return Ok(None);
    }
    let runs = build_runs(page, text)?;
    Ok(slip(&runs, page_width))
}

/// Rebuild pypdf-style text runs from Pdfium. pypdf's `visitor_text` fires once per text-showing
/// operator, with that operator's decoded string and text-matrix origin. Pdfium exposes that same
/// boundary as a *text object* (`FPDFText_GetTextObject`) — but only when the producer emits one
/// object per show operator. The two inputs here differ in exactly that:
///
/// * the fpdf2 testdata puts **one show operator per text object** (objects hold many chars);
///   grouping by text object then reproduces pypdf's runs directly, and is the only thing that
///   works: a value such as the `Qty Total` number is drawn *on top of* the trailing colon of
///   `Qty Total:` (their x positions overlap), so any left-to-right / gap grouping merges them and
///   even reorders the characters;
/// * the wkhtmltopdf samples put **one glyph per text object** (every object is single-char), so
///   the operator boundary is gone and runs must be rebuilt from glyph geometry.
///
/// So the page is inspected first: any multi-char text object means operator-level objects and the
/// run is one object; otherwise every object is a glyph and runs are rebuilt by geometry.
fn build_runs(page: &PdfPage, text: &PdfPageText) -> Result<Vec<Run>, String> {
    // Detect the granularity from the first few text objects only: the fpdf2 pages lead with a
    // multi-char object, and the wkhtmltopdf pages have no multi-char object at all. This keeps
    // the (quadratic) `chars_for_object` scan off the per-glyph pages.
    let mut per_glyph = true;
    let mut seen = 0usize;
    for obj in page.objects().iter() {
        if let Some(tobj) = obj.as_text_object() {
            if tobj.chars(text).map(|c| c.len() > 1).unwrap_or(false) {
                per_glyph = false;
                break;
            }
            seen += 1;
            if seen >= 16 {
                break;
            }
        }
    }
    if per_glyph {
        build_runs_geometry(text)
    } else {
        build_runs_objects(page, text)
    }
}

/// Operator-level objects (fpdf2 testdata): one run per text object, characters in Pdfium index
/// order, origin from the first glyph — pypdf's run, exactly.
fn build_runs_objects(page: &PdfPage, text: &PdfPageText) -> Result<Vec<Run>, String> {
    let mut runs: Vec<Run> = Vec::new();
    for (i, obj) in page.objects().iter().enumerate() {
        let tobj = match obj.as_text_object() {
            Some(t) => t,
            None => continue,
        };
        let chars = tobj
            .chars(text)
            .map_err(|e| format!("object {i} chars(): {e}"))?;
        let mut s = String::new();
        let mut origin: Option<(f64, f64)> = None;
        for c in chars.iter() {
            let raw = c.unicode_string().unwrap_or_default();
            for ch in raw.chars() {
                s.push(if ch == '\u{0002}' { '-' } else { ch });
            }
            if origin.is_none() && !raw.trim().is_empty() {
                if let Ok((x, y)) = c.origin() {
                    origin = Some((x.value as f64, y.value as f64));
                }
            }
        }
        let trimmed = s.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (x, y) = origin.unwrap_or((0.0, 0.0));
        runs.push(Run {
            x,
            y,
            text: trimmed.to_string(),
        });
    }
    Ok(runs)
}

/// Per-glyph objects (wkhtmltopdf samples): Pdfium's character order is not the visual order, so
/// characters are placed by their box first, then grouped: a run ends at a line change, a
/// horizontal gap wider than `SPLIT_POINTS`, or a font name/size change; a gap wider than
/// `SPACE_POINTS` inside a run becomes a space. A run's (x, y) is its first character's origin.
fn build_runs_geometry(text: &PdfPageText) -> Result<Vec<Run>, String> {
    const SPLIT_POINTS: f64 = 5.0;
    const SPACE_POINTS: f64 = 1.0;
    const LINE_TOLERANCE: f64 = 0.7;

    let chars = text.chars();
    let n = chars.len();
    // (baseline y, left, right, origin x, font, size, text); whitespace dropped.
    let mut items: Vec<(f64, f64, f64, f64, String, f32, String)> = Vec::new();
    let mut last_y = 0.0f64;
    for i in 0..n {
        let c = chars.get(i).map_err(|e| format!("chars.get({i}): {e}"))?;
        let raw = c.unicode_string().unwrap_or_default();
        // Pdfium reports a hyphen at a line end as U+0002; the PDF text is a real '-'.
        let s: String = raw
            .chars()
            .map(|ch| if ch == '\u{0002}' { '-' } else { ch })
            .collect();
        if s.trim().is_empty() {
            continue;
        }
        let (left, right) = match c.loose_bounds() {
            Ok(r) => (r.left().value as f64, r.right().value as f64),
            Err(_) => match c.tight_bounds() {
                Ok(r) => (r.left().value as f64, r.right().value as f64),
                Err(_) => continue,
            },
        };
        let (ox, oy) = match c.origin() {
            Ok((x, y)) => (x.value as f64, y.value as f64),
            Err(_) => (left, last_y),
        };
        if oy.is_finite() {
            last_y = oy;
        }
        items.push((
            oy,
            left,
            right,
            ox,
            c.font_name(),
            c.unscaled_font_size().value,
            s,
        ));
    }
    items.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(Ordering::Equal)
            .then(a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
    });

    let mut runs: Vec<Run> = Vec::new();
    let mut buf = String::new();
    let mut run_x = 0.0f64;
    let mut run_y = 0.0f64;
    let mut run_font = String::new();
    let mut run_size = 0.0f32;
    let mut last_right = 0.0f64;
    let mut have = false;
    for (oy, left, right, ox, font, size, s) in items {
        let gap = left - last_right;
        let new_run = !have
            || (oy - run_y).abs() > LINE_TOLERANCE
            || gap > SPLIT_POINTS
            || font != run_font
            || (size - run_size).abs() > 0.01;
        if new_run {
            if have {
                push_run(&mut runs, &mut buf, run_x, run_y);
                buf.clear();
            }
            run_x = ox;
            run_y = oy;
            run_font = font;
            run_size = size;
        } else if gap > SPACE_POINTS {
            buf.push(' ');
        }
        buf.push_str(&s);
        last_right = right;
        have = true;
    }
    if have {
        push_run(&mut runs, &mut buf, run_x, run_y);
    }
    Ok(runs)
}

fn push_run(runs: &mut Vec<Run>, buf: &mut String, x: f64, y: f64) {
    let trimmed = buf.trim();
    if !trimmed.is_empty() {
        runs.push(Run {
            x,
            y,
            text: trimmed.to_string(),
        });
    }
}

fn render_page(page: &PdfPage, dpi: f64) -> Result<(image::RgbaImage, i32, i32), PdfiumError> {
    let w = (page.width().value as f64 * dpi / 72.0).round() as i32;
    let h = (page.height().value as f64 * dpi / 72.0).round() as i32;
    let config = PdfRenderConfig::new()
        .set_target_width(w)
        .set_target_height(h);
    let bitmap = page.render_with_config(&config)?;
    let img = bitmap.as_image()?;
    Ok((img.into_rgba8(), w, h))
}

fn check_copy(
    pdfium: &Pdfium,
    docs: &[PdfDocument],
    files: &[PathBuf],
    order_pages: &[(String, Vec<(usize, usize)>)],
    out: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // choose 10 pages: the multi-page order first, then pages of file 0
    let mut chosen: Vec<(usize, usize)> = Vec::new();
    for (_, pages) in order_pages.iter() {
        if pages.len() >= 2 {
            for p in pages {
                if !chosen.contains(p) {
                    chosen.push(*p);
                }
            }
            break;
        }
    }
    let mut pi = 0usize;
    while chosen.len() < 10 && pi < docs[0].pages().len() as usize {
        let cand = (0usize, pi);
        if !chosen.contains(&cand) {
            chosen.push(cand);
        }
        pi += 1;
    }

    let mut dest = pdfium.create_new_pdf()?;
    for (k, (fi, pi)) in chosen.iter().enumerate() {
        dest.pages_mut()
            .copy_page_range_from_document(&docs[*fi], (*pi as PdfPageIndex)..=(*pi as PdfPageIndex), k as PdfPageIndex)?;
    }
    let copy_path = out.join("copy10.pdf");
    dest.save_to_file(&copy_path)?;

    let mut total_diff = 0usize;
    let mut max_diff = 0usize;
    let mut per_page = Vec::new();
    for (k, (fi, pi)) in chosen.iter().enumerate() {
        let src = docs[*fi].pages().get(*pi as PdfPageIndex)?;
        let dst = dest.pages().get(k as PdfPageIndex)?;
        let (src_img, sw, sh) = render_page(&src, DPI)?;
        let (dst_img, dw, dh) = render_page(&dst, DPI)?;
        if sw != dw || sh != dh {
            per_page.push((k, usize::MAX));
            continue;
        }
        let mut diff = 0usize;
        for (a, b) in src_img.pixels().zip(dst_img.pixels()) {
            if a != b {
                diff += 1;
            }
        }
        total_diff += diff;
        max_diff = max_diff.max(diff);
        src_img.save(out.join(format!("page-src-{k:02}.png")))?;
        dst_img.save(out.join(format!("page-dst-{k:02}.png")))?;
        per_page.push((k, diff));
    }
    let diffs: Vec<usize> = per_page.iter().map(|(_, d)| *d).collect();
    println!(
        "copy: pages={} out={} bytes  pixel_diff per page={:?} total={} max={}",
        chosen.len(),
        fs::metadata(&copy_path)?.len(),
        diffs,
        total_diff,
        max_diff
    );
    let _ = files;
    Ok(())
}

fn check_write(pdfium: &Pdfium, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // fonts.pdf: one A4 page, Arial / Arial bold / Consolas, made-up text
    let mut doc = pdfium.create_new_pdf()?;
    let arial = doc
        .fonts_mut()
        .load_true_type_from_file(r"C:\Windows\Fonts\arial.ttf", false)?;
    let arialbd = doc
        .fonts_mut()
        .load_true_type_from_file(r"C:\Windows\Fonts\arialbd.ttf", false)?;
    let consola = doc
        .fonts_mut()
        .load_true_type_from_file(r"C:\Windows\Fonts\consola.ttf", false)?;
    {
        let mut page = doc.pages_mut().create_page_at_end(PdfPagePaperSize::a4())?;
        let objects = page.objects_mut();
        let mut y = 760.0f32;
        let mut put = |x: f32, y: &mut f32, text: &str, font: &PdfFontToken, size: f32| {
            objects
                .create_text_object(
                    PdfPoints::new(x),
                    PdfPoints::new(*y),
                    text,
                    font.clone(),
                    PdfPoints::new(size),
                )
                .unwrap();
            *y -= size * 1.6;
        };
        put(56.0, &mut y, "Contoh dokumen uji — Arial biasa", &arial, 14.0);
        put(56.0, &mut y, "Contoh dokumen uji — Arial tebal", &arialbd, 14.0);
        y -= 10.0;
        for i in 0..6 {
            let line = format!("Baris {} · JY00000012{:02} · qty {}", i + 1, i, i + 1);
            put(56.0, &mut y, &line, &consola, 11.0);
        }
        y -= 6.0;
        put(56.0, &mut y, "Aa Bb Cc 0123456789 — contoh teks kecil", &arial, 9.0);
    }
    doc.save_to_file(&out.join("fonts.pdf"))?;

    // packing-list.pdf: 5 A4 pages, a made-up packing list
    write_packing_list(pdfium, out)?;

    println!(
        "fonts.pdf: {} bytes  packing-list.pdf: {} bytes",
        fs::metadata(out.join("fonts.pdf"))?.len(),
        fs::metadata(out.join("packing-list.pdf"))?.len()
    );
    Ok(())
}

fn write_packing_list(pdfium: &Pdfium, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = pdfium.create_new_pdf()?;
    let arial = doc
        .fonts_mut()
        .load_true_type_from_file(r"C:\Windows\Fonts\arial.ttf", false)?;
    let arialbd = doc
        .fonts_mut()
        .load_true_type_from_file(r"C:\Windows\Fonts\arialbd.ttf", false)?;
    let consola = doc
        .fonts_mut()
        .load_true_type_from_file(r"C:\Windows\Fonts\consola.ttf", false)?;

    const COLS: [f32; 6] = [40.0, 120.0, 250.0, 360.0, 430.0, 470.0];
    const ROWS_PER_PAGE: usize = 30;
    let names = [
        "Produk Contoh Alfa",
        "Produk Contoh Beta Panjang Sekali",
        "Produk Contoh Gamma",
        "Produk Contoh Delta",
    ];
    let variations = ["Contoh Varian Satu", "Contoh Varian Dua", ""];

    let mut row = 0usize;
    for pno in 0..5 {
        let mut page = doc.pages_mut().create_page_at_end(PdfPagePaperSize::a4())?;
        {
            let objects = page.objects_mut();
            objects.create_text_object(
                PdfPoints::new(COLS[0]),
                PdfPoints::new(800.0),
                format!("Packing list · 2026-10-07 · contoh halaman {}", pno + 1),
                arialbd.clone(),
                PdfPoints::new(13.0),
            )?;
            objects.create_text_object(
                PdfPoints::new(COLS[0]),
                PdfPoints::new(786.0),
                "5 contoh halaman · 150 baris contoh · 10:00",
                arial.clone(),
                PdfPoints::new(9.5),
            )?;

            let header = ["No", "Product Name", "Variation", "Seller SKU", "Qty", "Tracking ID"];
            for (c, word) in header.iter().enumerate() {
                objects.create_text_object(
                    PdfPoints::new(COLS[c]),
                    PdfPoints::new(765.0),
                    *word,
                    arialbd.clone(),
                    PdfPoints::new(9.0),
                )?;
            }
            objects.create_path_object_line(
                PdfPoints::new(COLS[0] - 6.0),
                PdfPoints::new(762.0),
                PdfPoints::new(560.0),
                PdfPoints::new(762.0),
                PdfColor::new(170, 170, 170, 255),
                PdfPoints::new(0.6),
            )?;

            for r in 0..ROWS_PER_PAGE {
                row += 1;
                let y = 748.0 - r as f32 * 22.0;
                objects.create_text_object(
                    PdfPoints::new(COLS[0]),
                    PdfPoints::new(y),
                    format!("{}", row),
                    arial.clone(),
                    PdfPoints::new(9.0),
                )?;
                objects.create_text_object(
                    PdfPoints::new(COLS[1]),
                    PdfPoints::new(y),
                    names[row % names.len()],
                    arial.clone(),
                    PdfPoints::new(9.0),
                )?;
                objects.create_text_object(
                    PdfPoints::new(COLS[2]),
                    PdfPoints::new(y),
                    variations[row % variations.len()],
                    arial.clone(),
                    PdfPoints::new(9.0),
                )?;
                objects.create_text_object(
                    PdfPoints::new(COLS[3]),
                    PdfPoints::new(y),
                    format!("SELL-CONT-{:03}", row),
                    arial.clone(),
                    PdfPoints::new(9.0),
                )?;
                objects.create_text_object(
                    PdfPoints::new(COLS[4]),
                    PdfPoints::new(y),
                    format!("{}", (row % 3) + 1),
                    consola.clone(),
                    PdfPoints::new(9.0),
                )?;
                objects.create_text_object(
                    PdfPoints::new(COLS[5]),
                    PdfPoints::new(y),
                    format!("JY{:010}", 1000 + row),
                    consola.clone(),
                    PdfPoints::new(9.0),
                )?;
            }
        }
    }
    doc.save_to_file(&out.join("packing-list.pdf"))?;
    Ok(())
}

fn write_pretty(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;
    file.write_all(serde_json::to_string_pretty(value)?.as_bytes())?;
    Ok(())
}
