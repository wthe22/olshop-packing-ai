//! The parcel's weight a label prints (reading.md, *The weight*): `Weight : 0.500 KG` on the TikTok
//! layouts (`Weight`, `5 KG` on the boxed one), `Berat: 500 gr` on the Shopee-style one, the value
//! on the caption's line. The instant labels and some others print none.

use crate::record::Weight;
use crate::text::PageText;

/// Each caption with the unit its value is printed in and the grams of that unit.
const CAPTIONS: [(&str, &str, u32); 2] = [("Weight", "KG", 1000), ("Berat:", "gr", 1)];
/// Runs this close to the caption's baseline are on its line (the Shopee-style value sits up to
/// 1.5 pt lower).
const SAME_LINE: f64 = 2.0;

/// The weight a label's first page prints. A product name of the slip may hold a caption's word too,
/// so every run that holds it is tried.
pub fn find_weight(page: &PageText) -> Weight {
    CAPTIONS
        .iter()
        .flat_map(|&caption| {
            page.runs
                .iter()
                .filter(move |run| run.text.contains(caption.0))
                .map(move |at| (caption, at))
        })
        .find_map(|((caption, unit, per_unit), at)| {
            page.runs
                .iter()
                .filter(|run| (run.y - at.y).abs() <= SAME_LINE)
                .find_map(|run| {
                    // The caption's run may hold the date printed left of it, or the value too.
                    let text = run
                        .text
                        .split_once(caption)
                        .map_or(&*run.text, |(_, after)| after);
                    grams(text.trim_start_matches([' ', ':']), unit, per_unit)
                })
        })
        .map_or(Weight::NotPrinted, Weight::Grams)
}

/// The grams of `0.500 KG` or `500 gr`: a number of up to three decimals and its unit.
fn grams(text: &str, unit: &str, per_unit: u32) -> Option<u32> {
    let number = text.trim().strip_suffix(unit)?.trim_end();
    let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty() || fraction.len() > 3 || !digits(whole) || !digits(fraction) {
        return None;
    }
    let thousandths: u32 = format!("{fraction:0<3}").parse().ok()?;
    let whole: u32 = whole.parse().ok()?;
    whole
        .checked_mul(per_unit)?
        .checked_add(thousandths * per_unit / 1000)
}

#[cfg(test)]
mod tests;
