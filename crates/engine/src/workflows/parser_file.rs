//! Reading `parser.toml` (rules-file/parser.md): the couriers, the clues that name them and the
//! header images that are not couriers.

use regex::Regex;
use serde::Deserialize;
use toml::Spanned;

use super::load::File;
use super::{Classifier, Courier, TrackingForm, WorkflowsError};
use crate::ids::CourierCode;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawParser {
    schema: Spanned<String>,
    #[serde(default)]
    couriers: Vec<RawCourier>,
    #[serde(default)]
    courier_classifier: Vec<RawClassifier>,
    #[serde(default)]
    not_courier: RawNotCourier,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCourier {
    code: Spanned<String>,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawClassifier {
    courier_code: Spanned<String>,
    #[serde(default)]
    logos: Vec<Spanned<String>>,
    #[serde(default)]
    shapes: Vec<Spanned<String>>,
    tracking_id: Option<Spanned<String>>,
    #[serde(default)]
    text: Vec<String>,
    #[serde(default)]
    instant_tags: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNotCourier {
    #[serde(default)]
    logos: Vec<Spanned<String>>,
}

/// What `parser.toml` holds, every check passed.
pub(super) struct Parser {
    pub(super) couriers: Vec<Courier>,
    pub(super) classifiers: Vec<Classifier>,
    pub(super) not_courier_logos: Vec<String>,
}

pub(super) fn read(file: &File) -> Result<Parser, WorkflowsError> {
    let raw: RawParser = file.parse()?;
    file.check_schema(&raw.schema)?;
    let couriers = file.couriers(&raw.couriers)?;
    let codes: Vec<&CourierCode> = couriers.iter().map(|courier| &courier.code).collect();
    let classifiers = file.classifiers(&raw.courier_classifier, &codes)?;
    let not_courier_logos = raw
        .not_courier
        .logos
        .iter()
        .map(|hash| file.hash(hash))
        .collect::<Result<_, _>>()?;
    Ok(Parser {
        couriers,
        classifiers,
        not_courier_logos,
    })
}

impl File<'_> {
    fn couriers(&self, raw: &[RawCourier]) -> Result<Vec<Courier>, WorkflowsError> {
        let mut couriers: Vec<Courier> = Vec::new();
        for entry in raw {
            let code = entry.code.get_ref();
            let valid = !code.is_empty()
                && code
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
            if !valid {
                return Err(self.error(
                    &entry.code.span(),
                    format!("courier code {code:?}: lower-case letters, digits and - only"),
                ));
            }
            if couriers.iter().any(|known| known.code.as_str() == code) {
                return Err(self.error(&entry.code.span(), format!("courier code {code:?} twice")));
            }
            couriers.push(Courier {
                code: CourierCode::new(code),
                name: entry.name.clone(),
            });
        }
        Ok(couriers)
    }

    fn classifiers(
        &self,
        raw: &[RawClassifier],
        codes: &[&CourierCode],
    ) -> Result<Vec<Classifier>, WorkflowsError> {
        let mut classifiers: Vec<Classifier> = Vec::new();
        for entry in raw {
            let code = entry.courier_code.get_ref();
            if !codes.iter().any(|known| known.as_str() == code) {
                return Err(self.error(
                    &entry.courier_code.span(),
                    format!("courier_code {code:?} names no courier"),
                ));
            }
            if classifiers
                .iter()
                .any(|known| known.courier.as_str() == code)
            {
                return Err(self.error(
                    &entry.courier_code.span(),
                    format!("a second courier_classifier for {code:?}"),
                ));
            }
            let tracking_id = match &entry.tracking_id {
                Some(form) => Some(self.tracking_form(form)?),
                None => None,
            };
            classifiers.push(Classifier {
                courier: CourierCode::new(code),
                logos: entry
                    .logos
                    .iter()
                    .map(|h| self.hash(h))
                    .collect::<Result<_, _>>()?,
                shapes: entry
                    .shapes
                    .iter()
                    .map(|h| self.hash(h))
                    .collect::<Result<_, _>>()?,
                tracking_id,
                text: entry.text.clone(),
                instant_tags: entry.instant_tags.clone(),
            });
        }
        Ok(classifiers)
    }

    fn hash(&self, hash: &Spanned<String>) -> Result<String, WorkflowsError> {
        let value = hash.get_ref();
        let valid = value.len() == 16
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if valid {
            Ok(value.clone())
        } else {
            Err(self.error(
                &hash.span(),
                format!("{value:?} is not a hash of 16 lower-case hex characters"),
            ))
        }
    }

    fn tracking_form(&self, form: &Spanned<String>) -> Result<TrackingForm, WorkflowsError> {
        let source = form.get_ref();
        let full = Regex::new(&format!("^(?:{source})$")).map_err(|error| {
            self.error(
                &form.span(),
                format!("tracking_id {source:?} is not a regex: {error}"),
            )
        })?;
        let unanchored =
            Regex::new(source).expect("a pattern that compiles anchored compiles bare");
        Ok(TrackingForm {
            source: source.clone(),
            full,
            unanchored,
        })
    }
}
