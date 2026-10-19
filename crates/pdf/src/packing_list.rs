//! The page builder for the three layouts (port of `tools/analysis/packing/packing_list.py`,
//! layouts `full`, `summary`, `pick`; 03 › *Packing list (A4)*, 07 › *Packing list*): the page
//! geometry in millimetres, the drawing operations a page is made of, and the rows the full and
//! summary layouts print.
//!
//! Geometry is ported in millimetres from fpdf2: a cell's text baseline sits at
//! `top + 0.5*h + 0.3*font_size` and a left-aligned cell indents its text by the 1 mm cell
//! margin (`fpdf.c_margin`). Wrapping width is the column width minus twice that margin.

mod emit;
mod fonts;
mod measure;
mod model;

pub use emit::write;
pub use model::{Layout, PackingLine, PackingList, RunOrder, RunRows, SavedPdfSection};

use packing_engine::ids::TrackingId;

use self::emit::grouped;
use self::fonts::Font;
use self::measure::{Measure, PT_PER_MM};

/// The message shown when a list has orders with no item lines (03 › *Packing list (A4)*).
const NO_ITEM_DATA: &str =
    "no item data: the packing list needs labels with packing slip or the orders CSV";

// Layout constants, millimetres (packing_list.py).
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 10.0;
const BOTTOM: f32 = PAGE_H - MARGIN; // 287
const CELL_MARGIN: f32 = 1.0; // fpdf c_margin
/// The page header's height: title 7 + counts line 5 + the 1 mm gap (packing_list.py `header`).
const HEADER_H: f32 = 13.0;
/// Where the body starts on every page — below the header, never at the top margin, or the first
/// row of the body is printed over the title and the counts line.
const BODY_TOP: f32 = MARGIN + HEADER_H; // 23
const LH: f32 = 5.2; // item line height
const TH: f32 = 4.6; // tracking-ID row height
const NCOL: usize = 6; // tracking-ID columns
const CW: f32 = 29.6; // tracking-ID column width
const X_ITEMS: f32 = 22.0; // item text left edge
const COL_X: [f32; 2] = [10.0, 106.0]; // pick-summary column left edges
const COL_W: f32 = 80.0; // pick-summary name wrap width
const ROW_H: f32 = 5.0; // pick-summary row height
const BAR_H: f32 = 6.5; // heading bar height

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
            y: BODY_TOP,
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
            self.y = BODY_TOP;
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
            .map(|order| order.tracking_id.as_ref().map_or("", TrackingId::as_str))
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
                self.y = BODY_TOP;
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
                self.y = BODY_TOP;
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

#[cfg(test)]
mod tests {
    use super::fonts::{cid_required, segments};

    /// Which characters are drawn with the CID font, and — the part the text layer depends on —
    /// that no CID run ever holds a character the CID font encodes wrongly (a plain space becomes
    /// U+00A0, `-` U+00AD, `·` U+A78F, so a run with one of those in it would spoil a viewer's
    /// text layer).
    #[test]
    fn only_the_characters_pdfium_encodes_wrongly_use_the_cid_font() {
        // The header, the counts line and plain names: one run, no CID font.
        for text in [
            "Packing list · 2026-10-11 · batch 1 · saved PDFs 1-2",
            "54 orders · 77 units · 9 runs · printed 13:15",
            "Sepatu Standar Samping Motor — honda",
            "Pick summary (units to take from stock)",
        ] {
            let parts = segments(text);
            assert_eq!(parts.len(), 1, "{text:?} is one run: {parts:?}");
            assert!(!parts[0].1, "{text:?} needs no CID font");
        }

        // An item line: only the `×` needs the CID font, the name and the count stay plain.
        assert_eq!(
            segments("Sepatu Standar Samping Motor — honda  ×1"),
            vec![
                ("Sepatu Standar Samping Motor — honda  ", false),
                ("×", true),
                ("1", false),
            ]
        );

        // Characters PDFium writes as `ÿ` each get their own run — the spaces between them stay
        // plain, and no run mixes the two.
        let parts = segments("Kabel Ñ ² ® ñ ½° ±");
        assert!(
            parts.len() > 3,
            "each such character is its own run: {parts:?}"
        );
        for (chunk, cid) in &parts {
            for ch in chunk.chars() {
                assert_eq!(
                    *cid,
                    cid_required(ch),
                    "{ch:?} in run {chunk:?} needs the CID font: {cid}"
                );
            }
        }
    }
}
