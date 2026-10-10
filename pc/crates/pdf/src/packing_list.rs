//! The three packing-list layouts (port of `script/packing/packing_list.py`, layouts `full`,
//! `summary`, `pick`; 03 › *Packing list (A4)*, 07 › *Packing list*).
//!
//! The input is the plain data model below, owned by this crate and derived from what
//! `packing_list.py` reads from `SavedPdf`/`Run` (orders, their slip lines and tracking IDs).
//! The engine's plan maps into it later, so this crate never depends on `plan.rs`.
//!
//! Geometry is ported in millimetres from fpdf2: a cell's text baseline sits at
//! `top + 0.5*h + 0.3*font_size` and a left-aligned cell indents its text by the 1 mm cell
//! margin (`fpdf.c_margin`). Wrapping width is the column width minus twice that margin.
//! PDFium writes each whole font into the file (~1.5 MB per batch), accepted in D2.

use std::fmt;
use std::path::Path;

use pdfium_render::prelude::*;

use crate::PdfError;

/// The font files, from `C:\Windows\Fonts` (03 › *Packing list (A4)*).
const FONT_DIR: &str = r"C:\Windows\Fonts";

/// The message shown when a list has orders with no item lines (03 › *Packing list (A4)*).
const NO_ITEM_DATA: &str =
    "no item data: the packing list needs labels with packing slip or the orders CSV";

// Layout constants, millimetres (packing_list.py).
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 10.0;
const BOTTOM: f32 = PAGE_H - MARGIN; // 287
const CELL_MARGIN: f32 = 1.0; // fpdf c_margin
const LH: f32 = 5.2; // item line height
const TH: f32 = 4.6; // tracking-ID row height
const NCOL: usize = 6; // tracking-ID columns
const CW: f32 = 29.6; // tracking-ID column width
const X_ITEMS: f32 = 22.0; // item text left edge
const COL_X: [f32; 2] = [10.0, 106.0]; // pick-summary column left edges
const COL_W: f32 = 80.0; // pick-summary name wrap width
const ROW_H: f32 = 5.0; // pick-summary row height
const BAR_H: f32 = 6.5; // heading bar height

const PT_PER_MM: f32 = 72.0 / 25.4;

fn to_pt(mm: f32) -> f32 {
    mm * PT_PER_MM
}

/// The three layouts (03 › *Packing list (A4)*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Item lines and the run's tracking IDs.
    Full,
    /// Item lines, no tracking IDs.
    Summary,
    /// Only the pick summary.
    Pick,
}

impl Layout {
    /// Parse a layout name; `None` for an unknown one.
    pub fn parse(name: &str) -> Option<Layout> {
        match name {
            "full" => Some(Layout::Full),
            "summary" => Some(Layout::Summary),
            "pick" => Some(Layout::Pick),
            _ => None,
        }
    }

    fn tracking(self) -> bool {
        self == Layout::Full
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Layout::Full => "full",
            Layout::Summary => "summary",
            Layout::Pick => "pick",
        })
    }
}

/// One slip line of an order as the packing list needs it (Python `orders.Line`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackingLine {
    pub name: String,
    /// Empty for the slip's `Default` variation.
    pub variation: String,
    pub quantity: i64,
    /// `orders.display_name(name, variation)`.
    pub display_name: String,
}

/// One order of a run (Python `orders.Order`): its tracking ID and slip lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunOrder {
    pub tracking_id: String,
    pub lines: Vec<PackingLine>,
}

/// One run of a saved PDF (Python `picks.Run`): a group of orders with identical contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunRows {
    /// The run number; the label is printed as `{section.number}-{number:02}`.
    pub number: u32,
    pub orders: Vec<RunOrder>,
}

/// One saved PDF (Python `picks.SavedPdf`): a pick's number, code, name and its runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedPdfSection {
    pub number: u32,
    pub code: String,
    pub name: String,
    pub runs: Vec<RunRows>,
}

/// The whole packing list: the day, the batch number (07 › *Packing list*) and the sections
/// (one for scope `per-pdf`, all for scope `whole`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackingList {
    /// `YYYY-MM-DD`.
    pub day: String,
    /// The batch number shown in the header, when the list belongs to one batch.
    pub batch: Option<u32>,
    /// Already-formatted `HH:MM` print time.
    pub printed: String,
    pub sections: Vec<SavedPdfSection>,
}

impl PackingList {
    fn orders(&self) -> usize {
        self.sections
            .iter()
            .flat_map(|section| &section.runs)
            .map(|run| run.orders.len())
            .sum()
    }

    fn units(&self) -> i64 {
        self.sections
            .iter()
            .flat_map(|section| &section.runs)
            .flat_map(|run| &run.orders)
            .flat_map(|order| &order.lines)
            .map(|line| line.quantity)
            .sum()
    }

    fn run_count(&self) -> usize {
        self.sections.iter().map(|section| section.runs.len()).sum()
    }
}

/// Write the packing list for `layout` and return the number of pages written. `pdfium` is the
/// library owned by the calling worker thread.
pub fn write(
    pdfium: &Pdfium,
    path: &Path,
    layout: Layout,
    list: &PackingList,
) -> Result<usize, PdfError> {
    if list.sections.is_empty() {
        return Err(PdfError::Message("no saved PDFs to write".to_string()));
    }
    // An order whose lines are unknown cannot be listed (Python raises here).
    for section in &list.sections {
        for run in &section.runs {
            for order in &run.orders {
                if order.lines.is_empty() {
                    return Err(PdfError::Message(NO_ITEM_DATA.to_string()));
                }
            }
        }
    }

    let subtitle = format!(
        "{} orders · {} units · {} runs · printed {}",
        grouped(list.orders() as i64),
        grouped(list.units()),
        grouped(list.run_count() as i64),
        list.printed
    );
    let title = title(list);

    let font_dir = Path::new(FONT_DIR);
    let measure = Measure::new(pdfium, font_dir)?;
    let pages = build_pages(layout, list, &title, &subtitle, &measure);
    emit(pdfium, path, &pages, font_dir)
}

/// Line 1 of the header: one saved PDF is named, several cover a number range
/// (03 › *Packing list (A4)*); the batch is added between (07 › *Packing list*).
fn title(list: &PackingList) -> String {
    let mut title = format!("Packing list · {}", list.day);
    if let Some(batch) = list.batch {
        title.push_str(&format!(" · batch {batch}"));
    }
    if list.sections.len() == 1 {
        let section = &list.sections[0];
        title.push_str(&format!(
            " · {}  {}  {}",
            section.number, section.code, section.name
        ));
    } else {
        let first = list.sections.first().map(|s| s.number).unwrap_or(0);
        let last = list.sections.last().map(|s| s.number).unwrap_or(0);
        title.push_str(&format!(" · saved PDFs {first}-{last}"));
    }
    title
}

/// Thousands separators, as Python's `{:,}`.
fn grouped(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}

// -------------------------------------------------------------------------------------------
// Measuring and layout
// -------------------------------------------------------------------------------------------

/// Which of the three packed fonts a piece of text uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Font {
    Arial,
    ArialBold,
    Consola,
}

fn font_file(font: Font) -> &'static str {
    match font {
        Font::Arial => "arial.ttf",
        Font::ArialBold => "arialbd.ttf",
        Font::Consola => "consola.ttf",
    }
}

/// Loads the three fonts into a document (PDFium embeds them whole).
fn load_fonts(
    doc: &mut PdfDocument<'_>,
    dir: &Path,
) -> Result<(PdfFontToken, PdfFontToken, PdfFontToken), PdfError> {
    let mut load = |file: &str| -> Result<PdfFontToken, PdfError> {
        let path = dir.join(file);
        doc.fonts_mut()
            .load_true_type_from_file(&path, false)
            .map_err(|e| PdfError::Font {
                path,
                message: e.to_string(),
            })
    };
    let arial = load(font_file(Font::Arial))?;
    let arial_bold = load(font_file(Font::ArialBold))?;
    let consola = load(font_file(Font::Consola))?;
    Ok((arial, arial_bold, consola))
}

/// One drawing operation, in millimetres from the page's top-left.
#[derive(Clone, Debug)]
enum Op {
    Text {
        x: f32,
        /// Baseline distance from the page top.
        baseline: f32,
        size_pt: f32,
        font: Font,
        text: String,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Fill {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
}

/// Measures text widths using a throwaway document with the same fonts (PDFium text objects
/// give the exact Arial/Consola advance widths fpdf2 used, so wrapping matches).
struct Measure<'a> {
    doc: PdfDocument<'a>,
    arial: PdfFontToken,
    arial_bold: PdfFontToken,
    consola: PdfFontToken,
}

impl<'a> Measure<'a> {
    fn new(pdfium: &'a Pdfium, dir: &Path) -> Result<Measure<'a>, PdfError> {
        let mut doc = pdfium.create_new_pdf()?;
        let (arial, arial_bold, consola) = load_fonts(&mut doc, dir)?;
        doc.pages_mut().create_page_at_end(PdfPagePaperSize::a4())?;
        Ok(Measure {
            doc,
            arial,
            arial_bold,
            consola,
        })
    }

    fn token(&self, font: Font) -> PdfFontToken {
        match font {
            Font::Arial => self.arial,
            Font::ArialBold => self.arial_bold,
            Font::Consola => self.consola,
        }
    }

    /// The advance width of `text` at `size_pt`, in millimetres.
    fn width_mm(&self, text: &str, font: Font, size_pt: f32) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let mut page = match self.doc.pages().get(0) {
            Ok(page) => page,
            Err(_) => return 0.0,
        };
        let objects = page.objects_mut();
        match objects.create_text_object(
            PdfPoints::new(0.0),
            PdfPoints::new(0.0),
            text,
            self.token(font),
            PdfPoints::new(size_pt),
        ) {
            Ok(object) => object.width().map(|w| w.value / PT_PER_MM).unwrap_or(0.0),
            Err(_) => 0.0,
        }
    }

    /// Greedy word wrap to `width_mm`, mirroring fpdf2 `multi_cell(..., dry_run=True)`.
    fn wrap(&self, text: &str, width_mm: f32, font: Font, size_pt: f32) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }
        let mut lines: Vec<String> = Vec::new();
        let mut current = String::new();
        for word in text.split(' ') {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            if current.is_empty() || self.width_mm(&candidate, font, size_pt) <= width_mm {
                current = candidate;
            } else {
                lines.push(std::mem::take(&mut current));
                current = word.to_string();
            }
        }
        lines.push(current);
        lines
    }
}

struct Builder<'m, 'a> {
    pages: Vec<Vec<Op>>,
    y: f32,
    layout: Layout,
    measure: &'m Measure<'a>,
}

impl<'m, 'a> Builder<'m, 'a> {
    fn new(layout: Layout, measure: &'m Measure<'a>) -> Builder<'m, 'a> {
        Builder {
            pages: vec![Vec::new()],
            y: MARGIN,
            layout,
            measure,
        }
    }

    fn ops(&mut self) -> &mut Vec<Op> {
        self.pages.last_mut().expect("at least one page")
    }

    /// A baseline for a cell of height `h` at top `y_top`.
    fn baseline(y_top: f32, h: f32, size_pt: f32) -> f32 {
        y_top + 0.5 * h + 0.3 * (size_pt / PT_PER_MM)
    }

    /// Left-aligned cell text (fpdf indents by the cell margin).
    fn cell(&mut self, x: f32, y_top: f32, h: f32, size_pt: f32, font: Font, text: &str) {
        let baseline = Self::baseline(y_top, h, size_pt);
        self.ops().push(Op::Text {
            x: x + CELL_MARGIN,
            baseline,
            size_pt,
            font,
            text: text.to_string(),
        });
    }

    /// Right-aligned text inside a cell at `x` of width `w`.
    #[allow(clippy::too_many_arguments)] // mirrors fpdf cell(x, w, y, h, size, font, text)
    fn cell_right(
        &mut self,
        x: f32,
        w: f32,
        y_top: f32,
        h: f32,
        size_pt: f32,
        font: Font,
        text: &str,
    ) {
        let width = self.measure.width_mm(text, font, size_pt);
        let baseline = Self::baseline(y_top, h, size_pt);
        self.ops().push(Op::Text {
            x: x + w - CELL_MARGIN - width,
            baseline,
            size_pt,
            font,
            text: text.to_string(),
        });
    }

    fn fill(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.ops().push(Op::Fill { x, y, w, h });
    }

    fn horizontal_line(&mut self, x1: f32, y: f32, x2: f32) {
        self.ops().push(Op::Line {
            x1,
            y1: y,
            x2,
            y2: y,
        });
    }

    /// Start a new page if `height` does not fit; repeat `continued` on the new page
    /// (fpdf2 `_need`). Returns whether a page break happened.
    fn need(&mut self, height: f32, continued: Option<&str>) -> bool {
        if self.y + height > BOTTOM {
            self.pages.push(Vec::new());
            self.y = MARGIN;
            if let Some(continued) = continued {
                self.cell(MARGIN, self.y, 5.0, 9.5, Font::Arial, continued);
                self.y += 5.0;
            }
            true
        } else {
            false
        }
    }

    fn draw_pdfs(&mut self, list: &PackingList) {
        for section in &list.sections {
            let orders: usize = section.runs.iter().map(|run| run.orders.len()).sum();
            let units: i64 = section
                .runs
                .iter()
                .flat_map(|run| &run.orders)
                .flat_map(|order| &order.lines)
                .map(|line| line.quantity)
                .sum();
            self.need(BAR_H + LH * 2.0, None);
            let top = self.y;
            self.fill(MARGIN, top, 190.0, BAR_H);
            self.cell(
                MARGIN,
                top,
                BAR_H,
                11.0,
                Font::ArialBold,
                &format!(" {}  {}  {}", section.number, section.code, section.name),
            );
            self.cell_right(
                160.0,
                40.0,
                top,
                BAR_H,
                9.5,
                Font::Arial,
                &format!(
                    "{} orders · {} units",
                    grouped(orders as i64),
                    grouped(units)
                ),
            );
            self.y = top + BAR_H;
            for run in &section.runs {
                self.draw_run(section, run);
            }
            self.y += 1.5;
        }
    }

    fn draw_run(&mut self, section: &SavedPdfSection, run: &RunRows) {
        let label = format!("{}-{:02}", section.number, run.number);
        let items = run_items(run);
        let tracking = self.layout.tracking();
        self.need(
            LH * items.len() as f32 + if tracking { TH + 1.0 } else { 0.0 } + 1.2,
            None,
        );
        let top = self.y;
        self.cell(MARGIN, top, LH, 11.5, Font::ArialBold, &label);
        for (index, text) in items.iter().enumerate() {
            self.cell(
                X_ITEMS,
                top + LH * index as f32,
                LH,
                10.5,
                Font::Arial,
                text,
            );
        }
        self.cell_right(
            180.0,
            20.0,
            top,
            LH,
            11.5,
            Font::ArialBold,
            &grouped(run.orders.len() as i64),
        );
        self.y = top + LH * items.len() as f32;
        if tracking {
            self.draw_tracking(&label, run, &items);
        }
        let line_y = self.y + 0.6;
        self.horizontal_line(MARGIN, line_y, 200.0);
        self.y = line_y + 0.6;
    }

    fn draw_tracking(&mut self, label: &str, run: &RunRows, items: &[String]) {
        let ids: Vec<&str> = run
            .orders
            .iter()
            .map(|order| order.tracking_id.as_str())
            .collect();
        let continued = format!("{label} (continued)  {}", items.join("  /  "));
        self.y += 0.6;
        for start in (0..ids.len()).step_by(NCOL) {
            self.need(TH, Some(&continued));
            let row_y = self.y;
            for (column, id) in ids[start..].iter().take(NCOL).enumerate() {
                self.cell(
                    X_ITEMS + column as f32 * CW,
                    row_y,
                    TH,
                    10.0,
                    Font::Consola,
                    id,
                );
            }
            self.y = row_y + TH;
        }
    }

    fn draw_pick_summary(&mut self, list: &PackingList) {
        let entries = pick_entries(list);
        if entries.is_empty() {
            return;
        }
        self.need(
            BAR_H + entries.len().min(4) as f32 * (ROW_H + 1.0) + 2.0,
            None,
        );
        self.y += 1.0;
        let top = self.y;
        self.fill(MARGIN, top, 190.0, BAR_H);
        self.cell(
            MARGIN,
            top,
            BAR_H,
            11.0,
            Font::ArialBold,
            " Pick summary (units to take from stock)",
        );
        self.y = top + BAR_H;
        let mut top = self.y;
        let mut index = 0;
        while index < entries.len() {
            let mut take = pick_fit(&entries[index..], BOTTOM - top, self.measure);
            if take == 0 {
                self.pages.push(Vec::new());
                self.y = MARGIN;
                top = self.y;
                take = pick_fit(&entries[index..], BOTTOM - top, self.measure).max(1);
            }
            let chunk = &entries[index..index + take];
            let half = chunk.len().div_ceil(2);
            for (column, part) in [&chunk[..half], &chunk[half..]].into_iter().enumerate() {
                let mut y = top;
                let x = COL_X[column];
                for (units, name) in part {
                    let lines =
                        self.measure
                            .wrap(name, COL_W - 2.0 * CELL_MARGIN, Font::Arial, 10.0);
                    self.cell_right(x, 12.0, y, ROW_H, 10.5, Font::ArialBold, &grouped(*units));
                    for (i, text) in lines.iter().enumerate() {
                        self.cell(
                            x + 14.0,
                            y + ROW_H * i as f32,
                            ROW_H,
                            10.0,
                            Font::Arial,
                            text,
                        );
                    }
                    y += ROW_H * lines.len() as f32 + 0.4;
                }
            }
            index += take;
            if index < entries.len() {
                self.pages.push(Vec::new());
                self.y = MARGIN;
                top = self.y;
            } else {
                self.y = top;
            }
        }
    }
}

/// The run's identical lines as display text, sorted (Python `_run_items`).
fn run_items(run: &RunRows) -> Vec<String> {
    let Some(first) = run.orders.first() else {
        return Vec::new();
    };
    let mut items: Vec<String> = first
        .lines
        .iter()
        .map(|line| format!("{}  ×{}", line.display_name, line.quantity))
        .collect();
    items.sort();
    items
}

/// One `(units, display name)` per `(name, variation)`, sorted by display name then key
/// (Python `_pick_entries`).
fn pick_entries(list: &PackingList) -> Vec<(i64, String)> {
    let mut units: Vec<((String, String), i64)> = Vec::new();
    let mut names: Vec<((String, String), String)> = Vec::new();
    for section in &list.sections {
        for run in &section.runs {
            for order in &run.orders {
                for line in &order.lines {
                    let key = (line.name.clone(), line.variation.clone());
                    match units.iter_mut().find(|(k, _)| *k == key) {
                        Some((_, value)) => *value += line.quantity,
                        None => units.push((key.clone(), line.quantity)),
                    }
                    if !names.iter().any(|(k, _)| *k == key) {
                        names.push((key, line.display_name.clone()));
                    }
                }
            }
        }
    }
    let name_of = |key: &(String, String)| -> String {
        names
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, name)| name.clone())
            .unwrap_or_default()
    };
    // Sort by (display name, name, variation), like Python's `sorted(units, key=...)`.
    let mut ordered: Vec<((String, String, String), i64, String)> = units
        .iter()
        .map(|(key, value)| {
            let display = name_of(key);
            (
                (display.clone(), key.0.clone(), key.1.clone()),
                *value,
                display,
            )
        })
        .collect();
    ordered.sort_by(|a, b| a.0.cmp(&b.0));
    ordered
        .into_iter()
        .map(|(_, value, display)| (value, display))
        .collect()
}

/// Largest count of the leading entries whose balanced two-column split fits `avail`
/// (Python `_pick_fit`).
fn pick_fit(entries: &[(i64, String)], avail: f32, measure: &Measure<'_>) -> usize {
    let mut heights: Vec<f32> = Vec::with_capacity(entries.len());
    for (_units, name) in entries {
        let lines = measure.wrap(name, COL_W - 2.0 * CELL_MARGIN, Font::Arial, 10.0);
        heights.push(ROW_H * lines.len() as f32 + 0.4);
    }
    let mut prefix = vec![0.0f32];
    for height in &heights {
        prefix.push(prefix.last().copied().unwrap_or(0.0) + height);
    }
    for count in (1..=entries.len()).rev() {
        let left = count.div_ceil(2);
        let right_total = prefix[count] - prefix[left];
        if prefix[left].max(right_total) <= avail {
            return count;
        }
    }
    0
}

// -------------------------------------------------------------------------------------------
// Emission
// -------------------------------------------------------------------------------------------

fn header_ops(
    title: &str,
    subtitle: &str,
    page_no: usize,
    pages: usize,
    measure: &Measure<'_>,
) -> Vec<Op> {
    let mut ops = Vec::new();
    let title_baseline = Builder::baseline(MARGIN, 7.0, 13.0);
    ops.push(Op::Text {
        x: MARGIN + CELL_MARGIN,
        baseline: title_baseline,
        size_pt: 13.0,
        font: Font::ArialBold,
        text: title.to_string(),
    });
    let pageno = format!("page {page_no}/{pages}");
    let width = measure.width_mm(&pageno, Font::Arial, 9.5);
    ops.push(Op::Text {
        x: 160.0 + 40.0 - CELL_MARGIN - width,
        baseline: Builder::baseline(MARGIN, 7.0, 9.5),
        size_pt: 9.5,
        font: Font::Arial,
        text: pageno,
    });
    ops.push(Op::Text {
        x: MARGIN + CELL_MARGIN,
        baseline: Builder::baseline(MARGIN + 7.0, 5.0, 9.5),
        size_pt: 9.5,
        font: Font::Arial,
        text: subtitle.to_string(),
    });
    ops
}

/// Lay the list out into page operations (header included, so the page numbers are known).
fn build_pages(
    layout: Layout,
    list: &PackingList,
    title: &str,
    subtitle: &str,
    measure: &Measure<'_>,
) -> Vec<Vec<Op>> {
    let mut builder = Builder::new(layout, measure);
    if layout != Layout::Pick {
        builder.draw_pdfs(list);
    }
    builder.draw_pick_summary(list);
    let body = builder.pages;
    let pages = body.len();
    body.into_iter()
        .enumerate()
        .map(|(index, ops)| {
            let mut page = header_ops(title, subtitle, index + 1, pages, measure);
            page.extend(ops);
            page
        })
        .collect()
}

/// Draw the laid-out pages into a new A4 PDF and save it.
fn emit(
    pdfium: &Pdfium,
    path: &Path,
    pages: &[Vec<Op>],
    font_dir: &Path,
) -> Result<usize, PdfError> {
    let mut doc = pdfium.create_new_pdf()?;
    let (arial, arial_bold, consola) = load_fonts(&mut doc, font_dir)?;
    let token = |font: Font| match font {
        Font::Arial => arial,
        Font::ArialBold => arial_bold,
        Font::Consola => consola,
    };
    for page_ops in pages {
        let mut page = doc.pages_mut().create_page_at_end(PdfPagePaperSize::a4())?;
        let objects = page.objects_mut();
        for op in page_ops {
            match op {
                Op::Text {
                    x,
                    baseline,
                    size_pt,
                    font,
                    text,
                } => {
                    objects.create_text_object(
                        PdfPoints::new(to_pt(*x)),
                        PdfPoints::new(to_pt(PAGE_H - *baseline)),
                        text,
                        token(*font),
                        PdfPoints::new(*size_pt),
                    )?;
                }
                Op::Line { x1, y1, x2, y2 } => {
                    objects.create_path_object_line(
                        PdfPoints::new(to_pt(*x1)),
                        PdfPoints::new(to_pt(PAGE_H - *y1)),
                        PdfPoints::new(to_pt(*x2)),
                        PdfPoints::new(to_pt(PAGE_H - *y2)),
                        PdfColor::new(170, 170, 170, 255),
                        PdfPoints::new(to_pt(0.2)),
                    )?;
                }
                Op::Fill { x, y, w, h } => {
                    let rect = PdfRect::new_from_values(
                        to_pt(PAGE_H - *y - *h),
                        to_pt(*x),
                        to_pt(PAGE_H - *y),
                        to_pt(*x + *w),
                    );
                    objects.create_path_object_rect(
                        rect,
                        None,
                        None,
                        Some(PdfColor::new(225, 225, 225, 255)),
                    )?;
                }
            }
        }
    }
    doc.save_to_file(path).map_err(|e| PdfError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    Ok(pages.len())
}
