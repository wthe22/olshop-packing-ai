//! Where each text of a label page goes. Positions are in points from the page's top left, as the
//! labels are laid out; `pdf.rs` turns them into PDF coordinates.
//!
//! The slip's columns and rows sit where the downloads have them. Texts on one line keep more than
//! 5 pt between them, or change font, as on the downloads, where the reader's run rule depends
//! on it.

use crate::metrics::{Font, text_width};
use crate::orders::{Label, Slip, Style};

pub const PAGE_WIDTH: f64 = 298.0;
pub const PAGE_HEIGHT: f64 = 420.0;
pub const FONT_SIZE: f64 = 9.0;

const LEFT: f64 = 5.8;
const SLIP_HEADER_Y: f64 = 290.0;
/// A wrapped line sits this much lower than the line above.
const LINE_STEP: f64 = 8.2;
const NAME_X: f64 = 5.8;
const VARIATION_X: f64 = 129.6;
const SELLER_SKU_X: f64 = 172.2;
const QTY_HEADER_X: f64 = 264.6;
const QTY_X: f64 = 268.6;
const QTY_TOTAL_X: f64 = 240.0;
const QTY_TOTAL_VALUE_X: f64 = 288.3;
const SLIP_ORDER_ID_X: f64 = 150.0;
const MESSAGE_X: f64 = 10.5;
const MESSAGE_COLON_X: f64 = 98.0;
const MESSAGE_VALUE_X: f64 = 110.0;
/// A cell wraps on spaces at this many characters.
const NAME_WRAP: usize = 25;
const VARIATION_WRAP: usize = 9;
const SELLER_SKU_WRAP: usize = 16;

/// One string at one place.
pub struct Text {
    pub font: Font,
    pub x: f64,
    pub top: f64,
    pub text: String,
}

pub struct Page {
    pub style: Style,
    pub texts: Vec<Text>,
}

/// The pages of one order: its label page, and a continuation page when its slip has a message.
pub fn label_pages(label: &Label) -> Vec<Page> {
    let mut texts = label_part(label);
    let mut pages = Vec::new();
    if let Some(slip) = &label.slip {
        texts.extend(slip_table(slip, &label.order_id));
        pages.push(Page {
            style: label.style,
            texts,
        });
        if let Some(message) = slip.message {
            pages.push(Page {
                style: label.style,
                texts: continuation(&label.order_id, message),
            });
        }
    } else {
        pages.push(Page {
            style: label.style,
            texts,
        });
    }
    pages
}

fn regular(x: f64, top: f64, text: impl Into<String>) -> Text {
    Text {
        font: Font::Regular,
        x,
        top,
        text: text.into(),
    }
}

fn bold(x: f64, top: f64, text: impl Into<String>) -> Text {
    Text {
        font: Font::Bold,
        x,
        top,
        text: text.into(),
    }
}

fn label_part(label: &Label) -> Vec<Text> {
    let put = |top: f64, value: String| {
        let value = if label.spaced { spaced(&value) } else { value };
        regular(LEFT, top, value)
    };
    let mut texts = vec![put(20.0, "Shipping Label".into())];
    texts.push(put(
        36.0,
        if label.spaced {
            format!("Order Id: {}", label.order_id)
        } else {
            label.order_id.clone()
        },
    ));
    if let Some(ship_by) = &label.ship_by {
        texts.push(put(52.0, format!("In transit by: {ship_by}")));
    }
    for index in 0..5 {
        texts.push(put(66.0 + index as f64 * 12.0, label.tracking_id.into()));
    }
    if label.prints_jet_site {
        texts.push(put(130.0, "www.jet.co.id".into()));
    }
    texts
}

fn spaced(value: &str) -> String {
    let chars: Vec<String> = value.chars().map(String::from).collect();
    chars.join(" ")
}

fn slip_header() -> Vec<Text> {
    vec![
        bold(NAME_X, SLIP_HEADER_Y, "Product Name"),
        bold(VARIATION_X, SLIP_HEADER_Y, "SKU"),
        bold(SELLER_SKU_X, SLIP_HEADER_Y, "Seller SKU"),
        bold(QTY_HEADER_X, SLIP_HEADER_Y, "Qty"),
    ]
}

fn slip_table(slip: &Slip, order_id: &str) -> Vec<Text> {
    let mut texts = slip_header();
    let mut top = SLIP_HEADER_Y + LINE_STEP;
    for row in &slip.rows {
        let cells = [
            (NAME_X, cell_lines(row.name, row.name_lines, NAME_WRAP)),
            (
                VARIATION_X,
                cell_lines(row.variation, row.variation_lines, VARIATION_WRAP),
            ),
            (
                SELLER_SKU_X,
                cell_lines(row.seller_sku, row.seller_sku_lines, SELLER_SKU_WRAP),
            ),
        ];
        for (column, (x, lines)) in cells.iter().enumerate() {
            for (index, line) in lines.iter().enumerate() {
                let at = top + index as f64 * LINE_STEP;
                let last_name_line = column == 0 && index + 1 == lines.len();
                match row
                    .bold_tail
                    .filter(|tail| last_name_line && line.len() > tail.len())
                {
                    Some(tail) if line.ends_with(tail) => {
                        let head = &line[..line.len() - tail.len()];
                        texts.push(regular(*x, at, head));
                        texts.push(bold(*x + text_width(Font::Regular, head), at, tail));
                    }
                    _ => texts.push(regular(*x, at, line.clone())),
                }
            }
        }
        texts.push(regular(QTY_X, top, row.quantity.to_string()));
        let height = cells
            .iter()
            .map(|(_, lines)| lines.len())
            .max()
            .unwrap_or(0);
        top += LINE_STEP * height.max(1) as f64;
    }
    top += LINE_STEP;
    texts.push(bold(QTY_TOTAL_X, top, "Qty Total:"));
    texts.push(regular(QTY_TOTAL_VALUE_X, top, slip.qty_total.to_string()));
    top += LINE_STEP;
    texts.push(regular(
        SLIP_ORDER_ID_X,
        top,
        format!("Order ID: {order_id}"),
    ));
    texts
}

fn continuation(order_id: &str, message: &str) -> Vec<Text> {
    let mut texts = slip_header();
    let top = SLIP_HEADER_Y + LINE_STEP;
    texts.push(regular(
        SLIP_ORDER_ID_X,
        top,
        format!("Order ID: {order_id}"),
    ));
    texts.push(bold(MESSAGE_X, top + LINE_STEP, "Customer Message"));
    texts.push(regular(MESSAGE_COLON_X, top + LINE_STEP, ":"));
    texts.push(regular(MESSAGE_VALUE_X, top + LINE_STEP, message));
    texts
}

/// A cell's printed lines: the fixed ones, else a greedy wrap on spaces, so that joining the lines
/// with a space gives the value back.
fn cell_lines(value: &str, fixed: Option<&[&str]>, limit: usize) -> Vec<String> {
    if let Some(lines) = fixed {
        return lines.iter().map(|line| line.to_string()).collect();
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in value.split(' ').filter(|word| !word.is_empty()) {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= limit {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::replace(&mut current, word.to_string()));
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::cell_lines;

    #[test]
    fn a_cell_wraps_on_spaces_at_its_width() {
        let cases: [(&str, usize, &[&str]); 4] = [
            ("", 9, &[]),
            ("Standard", 9, &["Standard"]),
            (
                "Sepatu Standar Samping Motor | Pelindung Rantai",
                25,
                &["Sepatu Standar Samping", "Motor | Pelindung Rantai"],
            ),
            (
                "Knalpot Racing Beat Full System",
                25,
                &["Knalpot Racing Beat Full", "System"],
            ),
        ];
        for (value, limit, want) in cases {
            assert_eq!(cell_lines(value, None, limit), want, "{value}");
        }
    }
}
