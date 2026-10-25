//! The courier deduction (courier.md): the tracking-ID form, the logo
//! lookup and the two-clue rule that names a courier.

use crate::ids::TrackingId;
use crate::text::LogoImage;

use super::fields::whitespace_re;
use super::{Courier, CourierTable, courier_data};

/// The candidate couriers of a tracking-ID form (courier.md item 2): every courier whose
/// `tracking` matches the whole of `tracking_id`.
pub fn candidates<'a>(table: &'a CourierTable, tracking_id: &TrackingId) -> Vec<&'a Courier> {
    table
        .couriers
        .iter()
        .filter(|courier| form_matches(courier, tracking_id))
        .collect()
}

/// Whether the tracking-ID form names `name` among its candidates (the two-clue rule's second
/// half; the order's `courier` is filled only where this holds).
pub fn form_names_courier_in(table: &CourierTable, tracking_id: &TrackingId, name: &str) -> bool {
    candidates(table, tracking_id)
        .iter()
        .any(|courier| courier.name == name)
}

fn form_matches(courier: &Courier, tracking_id: &TrackingId) -> bool {
    !tracking_id.as_str().is_empty()
        && courier
            .tracking_full
            .as_ref()
            .is_some_and(|re| re.is_match(tracking_id.as_str()))
}

/// The courier one header-band image names: its byte hash in a courier's `logo`, else its
/// appearance hash in a courier's `shape`. `None` for a hash answered *Not a courier* (a buying
/// channel's strip) and for an image the data does not know.
pub fn logo_courier<'a>(table: &'a CourierTable, image: &LogoImage) -> Option<&'a Courier> {
    if is_not_courier(
        table,
        Some(image.hash.as_str()),
        image.appearance.as_deref(),
    ) {
        return None;
    }
    if let Some(courier) = table
        .couriers
        .iter()
        .find(|courier| courier.logo.iter().any(|known| known == &image.hash))
    {
        return Some(courier);
    }
    let appearance = image.appearance.as_deref()?;
    table
        .couriers
        .iter()
        .find(|courier| courier.shape.iter().any(|known| known == appearance))
}

/// Whether the image is a band image the prompt must ask the owner about: a byte
/// hash not in `logo`/`not_courier` whose appearance is not in `shape` either.
pub fn is_unseen_logo(table: &CourierTable, image: &LogoImage) -> bool {
    if is_not_courier(
        table,
        Some(image.hash.as_str()),
        image.appearance.as_deref(),
    ) {
        return false;
    }
    if table
        .couriers
        .iter()
        .any(|courier| courier.logo.iter().any(|known| known == &image.hash))
    {
        return false;
    }
    !table.couriers.iter().any(|courier| {
        image
            .appearance
            .as_deref()
            .is_some_and(|appearance| courier.shape.iter().any(|known| known == appearance))
    })
}

fn is_not_courier(table: &CourierTable, hash: Option<&str>, appearance: Option<&str>) -> bool {
    table.not_courier.iter().any(|known| {
        hash.is_some_and(|hash| known == hash)
            || appearance.is_some_and(|appearance| known == appearance)
    })
}

/// The two-clue courier rule (courier.md items 1, 2, 4, 5): the tracking-ID form gives
/// the **candidate** couriers, and the courier is named only when the page's **logo** names one
/// of them (its byte hash, else its appearance hash). A logo whose courier and form disagree, a
/// logo with no form, a form with no logo, or neither → no name.
///
/// The printed text clue stands in for the logo only where the logo cannot be read at all — a
/// page with no header-band image — and must be a candidate too. On such a page a
/// form that carries the vendor's **own marker** (a lettered prefix: `JY…`, `TKP…`, `SPXID…`)
/// also names the courier on its own, when it names only one: that is the old `tracking_unique`
/// behaviour derived from the table instead of a flag, and it keeps a label whose image is
/// missing altogether (the test fixtures hold no images; a downloaded label always carries a
/// logo) readable. A **bare number** form (`00…`, `57…`) never names alone — the page is full of
/// numbers, so a number on its own is no clue, which is exactly what marking the `00` form
/// shared meant.
pub fn deduce_courier_in(
    table: &CourierTable,
    text: &str,
    tracking_id: Option<&TrackingId>,
    logos: &[LogoImage],
) -> Option<String> {
    let tracking_id = tracking_id?;
    let candidates = candidates(table, tracking_id);
    if candidates.is_empty() {
        return None;
    }

    if !logos.is_empty() {
        // Which courier the known band images name; two images naming different couriers cannot
        // be told apart, so nothing is named.
        let mut named: Option<&Courier> = None;
        for image in logos {
            let Some(courier) = logo_courier(table, image) else {
                continue;
            };
            match named {
                Some(other) if other.name != courier.name => return None,
                _ => named = Some(courier),
            }
        }
        return match named {
            Some(courier) if candidates.iter().any(|c| c.name == courier.name) => {
                Some(courier.name.clone())
            }
            _ => None,
        };
    }

    // No band image at all: the printed text clue stands in for the logo.
    let lowered = text.to_lowercase();
    let collapsed = whitespace_re().replace_all(text, "").to_lowercase();
    for courier in &candidates {
        if courier.text.iter().any(|clue| {
            let clue = clue.to_lowercase();
            lowered.contains(&clue) || collapsed.contains(&clue)
        }) {
            return Some(courier.name.clone());
        }
    }
    // A label whose image is missing altogether, and the form carries the vendor's own marker.
    if let [courier] = candidates.as_slice()
        && has_marker_prefix(courier)
    {
        return Some(courier.name.clone());
    }
    None
}

/// Whether the tracking form carries the vendor's own marker: its pattern begins with a letter
/// (`JY…`, `TKP…`, `SPXID…`) rather than a bare number (`00…`, `57…`).
fn has_marker_prefix(courier: &Courier) -> bool {
    courier
        .tracking
        .as_deref()
        .is_some_and(|pattern| pattern.starts_with(|c: char| c.is_ascii_alphabetic()))
}

/// The two-clue rule on the built-in `couriers.toml`.
pub fn deduce_courier(
    text: &str,
    tracking_id: Option<&TrackingId>,
    logos: &[LogoImage],
) -> Option<String> {
    deduce_courier_in(courier_data(), text, tracking_id, logos)
}
