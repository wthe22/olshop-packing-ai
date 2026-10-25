//! A record's moment: ISO 8601 with the UTC offset of the PC that wrote it
//! (`2026-10-07T14:26:00+07:00`), so a line says both the instant and the owner's clock.

use std::fmt;

use jiff::Zoned;
use jiff::fmt::strtime;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%:z";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Datetime(Zoned);

impl Datetime {
    /// The PC's clock now, to the second.
    pub fn now() -> Self {
        let now = Zoned::now();
        let second = now
            .round(jiff::Unit::Second)
            .expect("a second fits the clock's range");
        Self(second)
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let parsed = strtime::parse(FORMAT, text).map_err(|e| format!("{text:?}: {e}"))?;
        parsed
            .to_zoned()
            .map(Self)
            .map_err(|e| format!("{text:?}: {e}"))
    }

    pub fn zoned(&self) -> &Zoned {
        &self.0
    }
}

impl From<Zoned> for Datetime {
    fn from(zoned: Zoned) -> Self {
        Self(zoned)
    }
}

impl fmt::Display for Datetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.strftime(FORMAT))
    }
}

impl Serialize for Datetime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Datetime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Datetime::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Datetime;

    #[test]
    fn a_datetime_reads_back_as_written() {
        for text in ["2026-10-07T14:26:00+07:00", "2026-01-31T23:59:59-03:30"] {
            assert_eq!(Datetime::parse(text).unwrap().to_string(), text);
        }
        assert!(Datetime::parse("2026-10-07 14:26").is_err());
        assert!(Datetime::parse("2026-10-07T14:26:00Z").is_err());
    }
}
