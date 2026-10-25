//! Naming a copy's courier from its first page (courier.md), with the clue that named it: the
//! logo and the tracking-ID form must agree; text stands in for the logo only on a page with no
//! header image; an instant-service label is named by its service tag.

use std::sync::OnceLock;

use regex::Regex;

use crate::ids::{CourierCode, TrackingId};
use crate::label::whitespace_re;
use crate::record::{CourierClue, CourierClueKind};
use crate::text::{LogoImage, PageText};
use crate::workflows::{Classifier, Workflows};

/// How far down from the top of the page the header band reaches, in points.
const BAND_POINTS: f64 = 40.0;

/// The courier of a page and the clue that named it; `None` and a clue of kind `none` when the
/// clues do not agree.
pub(super) fn name_courier(
    workflows: &Workflows,
    page: &PageText,
    tracking_id: Option<&TrackingId>,
) -> (Option<CourierCode>, CourierClue) {
    let none = (None, clue(CourierClueKind::None, None, None));
    let named_logo = logo_courier(workflows, &page.logos);
    if tracking_id.is_none() && is_instant(&page.text) {
        if let Some(Named::Courier(..)) = named_logo {
            return none;
        }
        return instant_courier(workflows, page).unwrap_or(none);
    }
    let Some(tracking_id) = tracking_id else {
        return none;
    };
    let candidates: Vec<&Classifier> = workflows
        .classifiers
        .iter()
        .filter(|c| {
            c.tracking_id
                .as_ref()
                .is_some_and(|f| f.matches(tracking_id.as_str()))
        })
        .collect();
    let form = |classifier: &Classifier| {
        classifier
            .tracking_id
            .as_ref()
            .map(|form| form.source.clone())
    };
    if !page.logos.is_empty() {
        return match named_logo {
            Some(Named::Courier(classifier, kind, value)) => candidates
                .iter()
                .find(|c| c.courier == classifier.courier)
                .map_or(none.clone(), |c| {
                    (Some(c.courier.clone()), clue(kind, Some(value), form(c)))
                }),
            _ => none,
        };
    }
    // No header image at all: printed text stands in for the logo.
    let lowered = page.text.to_lowercase();
    let collapsed = whitespace_re().replace_all(&page.text, "").to_lowercase();
    for classifier in &candidates {
        if let Some(text) = classifier.text.iter().find(|text| {
            let text = text.to_lowercase();
            lowered.contains(&text) || collapsed.contains(&text)
        }) {
            let found = clue(CourierClueKind::Text, Some(text.clone()), form(classifier));
            return (Some(classifier.courier.clone()), found);
        }
    }
    // A lettered form that fits one courier names it; a bare-number form never does.
    if let [only] = candidates.as_slice() {
        let lettered =
            form(only).is_some_and(|source| source.starts_with(|c: char| c.is_ascii_alphabetic()));
        if lettered {
            return (
                Some(only.courier.clone()),
                clue(CourierClueKind::Form, None, form(only)),
            );
        }
    }
    none
}

fn clue(kind: CourierClueKind, value: Option<String>, form: Option<String>) -> CourierClue {
    CourierClue { kind, value, form }
}

/// What the page's header images say.
enum Named<'a> {
    /// One courier, by the bytes (`logo`) or the appearance (`shape`) of its logo.
    Courier(&'a Classifier, CourierClueKind, String),
    /// Two images naming different couriers: nothing can be named.
    Conflict,
}

fn logo_courier<'a>(workflows: &'a Workflows, logos: &[LogoImage]) -> Option<Named<'a>> {
    let mut named: Option<Named<'a>> = None;
    for image in logos {
        let not_courier = workflows
            .not_courier_logos
            .iter()
            .any(|known| known == &image.hash || image.appearance.as_ref() == Some(known));
        if not_courier {
            continue;
        }
        let by_bytes = workflows
            .classifiers
            .iter()
            .find(|c| c.logos.contains(&image.hash))
            .map(|c| (c, CourierClueKind::Logo, image.hash.clone()));
        let found = by_bytes.or_else(|| {
            let appearance = image.appearance.as_ref()?;
            workflows
                .classifiers
                .iter()
                .find(|c| c.shapes.contains(appearance))
                .map(|c| (c, CourierClueKind::Shape, appearance.clone()))
        });
        let Some((classifier, kind, value)) = found else {
            continue;
        };
        named = match named {
            Some(Named::Courier(other, ..)) if other.courier != classifier.courier => {
                Some(Named::Conflict)
            }
            Some(Named::Conflict) => Some(Named::Conflict),
            _ => Some(Named::Courier(classifier, kind, value)),
        };
    }
    named
}

fn pickup_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^[A-Z]{2}\d{4}$").expect("built-in pickup regex"))
}

/// How far from the caption's line the pickup code may sit, in points: it is printed beside it
/// in large type.
const PICKUP_REACH: f64 = 30.0;

/// An instant-service label prints `Kode Pengambilan` (business-process 02).
fn is_instant(text: &str) -> bool {
    whitespace_re()
        .replace_all(text, "")
        .contains("KodePengambilan")
}

/// The pickup code of an instant-service label: the run of two capital letters and four digits
/// beside its `Kode Pengambilan` caption. The page text does not keep the two together.
pub(super) fn pickup_code(page: &PageText) -> Option<String> {
    let caption = page.runs.iter().find(|run| {
        whitespace_re()
            .replace_all(&run.text, "")
            .contains("KodePengambilan")
    })?;
    page.runs
        .iter()
        .find(|run| {
            (run.y - caption.y).abs() <= PICKUP_REACH && pickup_re().is_match(run.text.trim())
        })
        .map(|run| run.text.trim().to_string())
}

/// The courier whose `instant_tags` hold a word of the page's header band.
fn instant_courier(
    workflows: &Workflows,
    page: &PageText,
) -> Option<(Option<CourierCode>, CourierClue)> {
    let band = page.height - BAND_POINTS;
    let words = page
        .runs
        .iter()
        .filter(|run| run.y >= band)
        .flat_map(|run| run.text.split_whitespace());
    for word in words {
        if let Some(classifier) = workflows
            .classifiers
            .iter()
            .find(|c| c.instant_tags.iter().any(|tag| tag == word))
        {
            let found = clue(CourierClueKind::Tag, Some(word.to_string()), None);
            return Some((Some(classifier.courier.clone()), found));
        }
    }
    None
}
