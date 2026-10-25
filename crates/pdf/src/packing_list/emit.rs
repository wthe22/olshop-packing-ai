//! Writing the packing list: the entry point that checks the list and names it, the page header,
//! the pages laid out into operations, and the new A4 PDF they are drawn into.

use std::path::Path;

use pdfium_render::prelude::*;

use crate::PdfError;

use super::Layout;
use super::fonts::{FONT_DIR, Font, FontSet, cid_required};
use super::measure::{Measure, to_pt};
use super::model::PackingList;
use super::{Builder, CELL_MARGIN, MARGIN, NO_ITEM_DATA, Op, PAGE_H};

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
    // An order whose lines are unknown cannot be listed.
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
    emit(pdfium, path, &pages, font_dir, &measure)
}

/// Line 1 of the header: one saved PDF is named, several cover a number range, and the batch
/// is printed between them.
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
        let first = list.sections.first().map(|s| s.number.to_string());
        let last = list.sections.last().map(|s| s.number.to_string());
        let (first, last) = (first.unwrap_or_default(), last.unwrap_or_default());
        title.push_str(&format!(" · saved PDFs {first}-{last}"));
    }
    title
}

/// Thousands separated by commas: `1,234`.
pub(super) fn grouped(n: i64) -> String {
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

pub(super) fn header_ops(
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
pub(super) fn build_pages(
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
    builder.draw_category_summary(list);
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
pub(super) fn emit(
    pdfium: &Pdfium,
    path: &Path,
    pages: &[Vec<Op>],
    font_dir: &Path,
    measure: &Measure<'_>,
) -> Result<usize, PdfError> {
    let mut doc = pdfium.create_new_pdf()?;
    let mut fonts = FontSet::load(&mut doc, font_dir)?;
    load_cid_fonts(&mut doc, &mut fonts, pages, font_dir)?;
    for page_ops in pages {
        draw_page(&mut doc, page_ops, &fonts, measure)?;
    }
    doc.save_to_file(path).map_err(|e| PdfError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    Ok(pages.len())
}

/// The CID flavour is embedded only for the fonts a page really needs, so a list whose text has no
/// such character keeps the smaller file.
fn load_cid_fonts(
    doc: &mut PdfDocument<'_>,
    fonts: &mut FontSet,
    pages: &[Vec<Op>],
    font_dir: &Path,
) -> Result<(), PdfError> {
    for font in [Font::Arial, Font::ArialBold, Font::Consola] {
        let needed = pages.iter().flatten().any(|op| match op {
            Op::Text {
                font: used, text, ..
            } => *used == font && text.chars().any(cid_required),
            _ => false,
        });
        if needed {
            fonts.load_cid(doc, font_dir, font)?;
        }
    }
    Ok(())
}

/// Draw one page's operations into a new A4 page.
fn draw_page(
    doc: &mut PdfDocument<'_>,
    page_ops: &[Op],
    fonts: &FontSet,
    measure: &Measure<'_>,
) -> Result<(), PdfError> {
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
                for (offset, chunk, cid) in measure.segments_with_offsets(text, *font, *size_pt) {
                    objects.create_text_object(
                        PdfPoints::new(to_pt(*x + offset)),
                        PdfPoints::new(to_pt(PAGE_H - *baseline)),
                        chunk,
                        fonts.token(*font, cid),
                        PdfPoints::new(*size_pt),
                    )?;
                }
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
    Ok(())
}
