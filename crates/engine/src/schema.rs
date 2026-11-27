//! Schema versions `X.Y` (data/README.md, *Types, versions and names*): a copy reads every file of
//! its own X and passes over what it does not know; a file of another X it does not read.

use std::fmt;

use serde::{Serialize, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Schema {
    major: u16,
    minor: u16,
}

/// How a file's `schema` stands to the one this copy writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaFit {
    /// The same X: read, whatever its Y.
    Reads,
    /// A lower X: read only by the owner's migration.
    Older,
    /// A higher X: a newer copy's, not read.
    Newer,
}

impl Schema {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// `X.Y`, each a whole number; anything else is none.
    pub fn parse(text: &str) -> Option<Self> {
        let (major, minor) = text.split_once('.')?;
        let number = |part: &str| {
            (!part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
                .then(|| part.parse().ok())
                .flatten()
        };
        Some(Self::new(number(major)?, number(minor)?))
    }

    /// How `found` stands to `self`, the version this copy writes.
    pub fn fit(self, found: Schema) -> SchemaFit {
        match found.major.cmp(&self.major) {
            std::cmp::Ordering::Equal => SchemaFit::Reads,
            std::cmp::Ordering::Less => SchemaFit::Older,
            std::cmp::Ordering::Greater => SchemaFit::Newer,
        }
    }

    /// `2.x`: every version this copy reads.
    pub fn family(self) -> String {
        format!("{}.x", self.major)
    }
}

/// Whether app version `found` is newer than `ours` (`0.8.0-alpha.10` after `0.8.0-alpha.9`, `0.8.0`
/// after both), by semantic version precedence; a version that does not read is not newer.
pub fn newer_app(found: &str, ours: &str) -> bool {
    match (AppVersion::parse(found), AppVersion::parse(ours)) {
        (Some(found), Some(ours)) => found > ours,
        _ => false,
    }
}

/// An app version in semantic version order: the numbers, then a pre-release below its release.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AppVersion {
    numbers: Vec<u64>,
    stage: Stage,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    PreRelease(Vec<Identifier>),
    Release,
}

/// A pre-release identifier: a number sorts below a word, numbers by value, words by text.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Identifier {
    Number(u64),
    Word(String),
}

impl AppVersion {
    fn parse(text: &str) -> Option<Self> {
        let (numbers, pre) = match text.split_once('-') {
            Some((numbers, pre)) => (numbers, Some(pre)),
            None => (text, None),
        };
        let mut numbers: Vec<u64> = numbers
            .split('.')
            .map(|part| part.parse().ok())
            .collect::<Option<_>>()?;
        let mut identifiers: Vec<Identifier> = pre
            .into_iter()
            .flat_map(|pre| pre.split('.'))
            .map(|part| match part.parse() {
                Ok(number) => Identifier::Number(number),
                Err(_) => Identifier::Word(part.to_string()),
            })
            .collect();
        // Copies of 0.8 wrote `0.8.N-alpha` for `0.8.0-alpha.N`, and journals keep it.
        if numbers.len() == 3
            && numbers[..2] == [0, 8]
            && numbers[2] > 0
            && identifiers == [Identifier::Word("alpha".into())]
        {
            identifiers.push(Identifier::Number(numbers[2]));
            numbers[2] = 0;
        }
        let stage = match pre {
            Some(_) => Stage::PreRelease(identifiers),
            None => Stage::Release,
        };
        Some(Self { numbers, stage })
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Serialize for Schema {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_version_is_two_whole_numbers() {
        let cases = [
            ("2.0", Some(Schema::new(2, 0))),
            ("2.10", Some(Schema::new(2, 10))),
            ("10.1", Some(Schema::new(10, 1))),
            ("2", None),
            ("2.", None),
            (".1", None),
            ("2.1.0", None),
            ("v2.1", None),
            ("2.+1", None),
            ("", None),
        ];
        for (text, want) in cases {
            assert_eq!(Schema::parse(text), want, "{text:?}");
        }
    }

    #[test]
    fn a_copy_reads_every_version_of_its_major() {
        let ours = Schema::new(2, 1);
        let cases = [
            ("2.0", SchemaFit::Reads),
            ("2.1", SchemaFit::Reads),
            ("2.7", SchemaFit::Reads),
            ("1.9", SchemaFit::Older),
            ("3.0", SchemaFit::Newer),
        ];
        for (text, want) in cases {
            assert_eq!(ours.fit(Schema::parse(text).unwrap()), want, "{text}");
        }
        assert!(Schema::new(2, 2) > ours && Schema::new(10, 0) > Schema::new(9, 9));
    }

    #[test]
    fn an_app_version_is_newer_by_semantic_version_order() {
        let cases = [
            ("0.8.0-alpha.10", "0.8.0-alpha.9", true),
            ("0.8.0-alpha.9", "0.8.0-alpha.10", false),
            ("0.8.0-alpha.1", "0.8.0-alpha", true),
            ("0.8.0-beta", "0.8.0-alpha.10", true),
            ("0.8.0", "0.8.0-alpha.10", true),
            ("0.9.0", "0.8.0", true),
            ("0.10.0", "0.9.1", true),
            ("0.8.0-alpha.10", "0.8.0-alpha.10", false),
            // The journals' `0.8.N-alpha` is `0.8.0-alpha.N`.
            ("0.8.9-alpha", "0.8.0-alpha.10", false),
            ("0.8.9-alpha", "0.8.0-alpha.8", true),
            ("0.8.0-alpha.10", "0.8.9-alpha", true),
            ("0.8.0-alpha", "0.8.1-alpha", false),
            ("next", "0.8.0", false),
        ];
        for (found, ours, want) in cases {
            assert_eq!(newer_app(found, ours), want, "{found} after {ours}");
        }
    }

    #[test]
    fn it_reads_and_writes_as_x_dot_y() {
        assert_eq!(Schema::new(2, 1).to_string(), "2.1");
        assert_eq!(Schema::new(2, 1).family(), "2.x");
        assert_eq!(
            serde_json::to_string(&Schema::new(1, 0)).unwrap(),
            "\"1.0\""
        );
    }
}
