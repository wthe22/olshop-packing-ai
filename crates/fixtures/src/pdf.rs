//! The pages drawn into a PDF with `lopdf`. A `Glyphs` page puts each character in a text object
//! of its own at its own place, spaces left out, as the downloads are made; a `Words` page puts
//! each string in one object.

use std::path::Path;

use lopdf::content::{Content, Operation};
use lopdf::{Dictionary, Document, Object, Stream, StringFormat, dictionary};

use crate::draw::{FONT_SIZE, PAGE_HEIGHT, PAGE_WIDTH, Page, Text};
use crate::metrics::{FIRST_CHAR, Font, LAST_CHAR};
use crate::orders::Style;

/// Write `pages` as one PDF at `path`. The bytes depend on the pages only.
pub fn write(path: &Path, pages: &[Page]) -> std::io::Result<()> {
    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let mut fonts = Dictionary::new();
    for font in [Font::Regular, Font::Bold] {
        let id = doc.add_object(font_dictionary(font));
        fonts.set(font.resource_name(), id);
    }
    let resources_id = doc.add_object(dictionary! { "Font" => fonts });
    let mut kids = Vec::with_capacity(pages.len());
    for page in pages {
        let content = Content {
            operations: operations(page),
        };
        let bytes = content.encode().map_err(std::io::Error::other)?;
        let content_id = doc.add_object(Stream::new(dictionary! {}, bytes));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), real(PAGE_WIDTH), real(PAGE_HEIGHT)],
            "Contents" => content_id,
            "Resources" => resources_id,
        });
        kids.push(page_id.into());
    }
    let count = kids.len() as i64;
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => kids, "Count" => count }),
    );
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    doc.compress();
    doc.save(path).map(|_| ())
}

fn font_dictionary(font: Font) -> Dictionary {
    let widths: Vec<Object> = font
        .widths()
        .iter()
        .map(|width| Object::Integer(*width as i64))
        .collect();
    dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => font.base_name(),
        "Encoding" => "WinAnsiEncoding",
        "FirstChar" => FIRST_CHAR as i64,
        "LastChar" => LAST_CHAR as i64,
        "Widths" => widths,
    }
}

fn real(value: f64) -> Object {
    Object::Real(value as f32)
}

fn operations(page: &Page) -> Vec<Operation> {
    let mut operations = Vec::new();
    for text in &page.texts {
        match page.style {
            Style::Glyphs => {
                let mut x = text.x;
                for ch in text.text.chars() {
                    if ch != ' ' {
                        operations.extend(text_object(text, x, ch.to_string()));
                    }
                    x += text.font.advance(ch, FONT_SIZE);
                }
            }
            Style::Words => operations.extend(text_object(text, text.x, text.text.clone())),
        }
    }
    operations
}

/// `BT /F 9 Tf 1 0 0 1 x y Tm (s) Tj ET`: one text object at (x, the baseline from the top).
fn text_object(text: &Text, x: f64, string: String) -> [Operation; 5] {
    let y = PAGE_HEIGHT - text.top;
    [
        Operation::new("BT", vec![]),
        Operation::new(
            "Tf",
            vec![text.font.resource_name().into(), real(FONT_SIZE)],
        ),
        Operation::new(
            "Tm",
            vec![1.into(), 0.into(), 0.into(), 1.into(), real(x), real(y)],
        ),
        Operation::new(
            "Tj",
            vec![Object::String(string.into_bytes(), StringFormat::Literal)],
        ),
        Operation::new("ET", vec![]),
    ]
}
