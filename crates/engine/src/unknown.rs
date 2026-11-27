//! What a reader passes over: the keys of a TOML file and the fields of a JSON record this version
//! does not know (data/README.md, *Types, versions and names*). They stay in the file; this only
//! finds them, so the owner can look.

use std::fmt;

use serde::de::DeserializeOwned;
use serde_json::Value;
use toml_edit::{Document, Item, Table};

/// A key of a TOML file this version does not read, with its line when it can be found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownKey {
    /// From the top: `label_sort_rules[1].enabeld`.
    pub path: String,
    pub line: Option<usize>,
}

impl fmt::Display for UnknownKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}", self.path),
            None => f.write_str(&self.path),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Step {
    Key(String),
    Index(usize),
}

/// The keys of `text` a reader of shape `T` passes over; none when the text does not read as `T`
/// (its own check names that).
pub fn unknown_toml_keys<T: DeserializeOwned>(text: &str) -> Vec<UnknownKey> {
    let Ok(deserializer) = toml::Deserializer::parse(text) else {
        return Vec::new();
    };
    let mut paths: Vec<Vec<Step>> = Vec::new();
    let read: Result<T, _> = serde_ignored::deserialize(deserializer, |path| {
        paths.push(steps(&path));
    });
    if read.is_err() {
        return Vec::new();
    }
    let document = Document::parse(text).ok();
    let mut keys = paths
        .into_iter()
        .map(|path| UnknownKey {
            line: document
                .as_ref()
                .and_then(|doc| key_offset(doc.as_table(), &path))
                .map(|offset| line_of(text, offset)),
            path: path_text(&path),
        })
        .collect::<Vec<_>>();
    keys.sort_by_key(|key| key.line);
    keys
}

/// The fields of `found` that `known` (the same record read and written back by this version) does
/// not have, as paths: `orders[3].colour`.
pub fn unknown_json_fields(found: &Value, known: &Value) -> Vec<String> {
    let mut fields = Vec::new();
    walk(found, known, &mut Vec::new(), &mut fields);
    fields
}

fn walk(found: &Value, known: &Value, path: &mut Vec<Step>, fields: &mut Vec<String>) {
    match (found, known) {
        (Value::Object(found), Value::Object(known)) => {
            for (key, value) in found {
                path.push(Step::Key(key.clone()));
                match known.get(key) {
                    Some(known) => walk(value, known, path, fields),
                    None => fields.push(path_text(path)),
                }
                path.pop();
            }
        }
        (Value::Array(found), Value::Array(known)) => {
            for (index, (value, known)) in found.iter().zip(known).enumerate() {
                path.push(Step::Index(index));
                walk(value, known, path, fields);
                path.pop();
            }
        }
        _ => {}
    }
}

fn steps(path: &serde_ignored::Path) -> Vec<Step> {
    use serde_ignored::Path;
    let (parent, step) = match path {
        Path::Root => return Vec::new(),
        Path::Seq { parent, index } => (parent, Some(Step::Index(*index))),
        Path::Map { parent, key } => (parent, Some(Step::Key(key.clone()))),
        Path::Some { parent }
        | Path::NewtypeStruct { parent }
        | Path::NewtypeVariant { parent } => (parent, None),
    };
    let mut steps = steps(parent);
    steps.extend(step);
    steps
}

fn path_text(path: &[Step]) -> String {
    let mut text = String::new();
    for step in path {
        match step {
            Step::Key(key) if text.is_empty() => text.push_str(key),
            Step::Key(key) => text.push_str(&format!(".{key}")),
            Step::Index(index) => text.push_str(&format!("[{index}]")),
        }
    }
    text
}

/// Where the last key of `path` is written, following tables and arrays of tables.
fn key_offset(table: &Table, path: &[Step]) -> Option<usize> {
    let (Step::Key(key), rest) = path.split_first()? else {
        return None;
    };
    if rest.is_empty() {
        return table.key(key)?.span().map(|span| span.start);
    }
    match (table.get(key)?, rest) {
        (Item::Table(inner), _) => key_offset(inner, rest),
        (Item::ArrayOfTables(array), [Step::Index(index), rest @ ..]) => {
            key_offset(array.get(*index)?, rest)
        }
        _ => None,
    }
}

fn line_of(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;

    use super::*;

    #[derive(Deserialize)]
    struct Root {
        schema: String,
        #[serde(default)]
        rules: Vec<Rule>,
        sub: Option<Sub>,
    }

    #[derive(Deserialize)]
    struct Rule {
        name: String,
    }

    #[derive(Deserialize)]
    struct Sub {
        on: bool,
    }

    #[test]
    fn a_toml_key_no_field_reads_is_named_with_its_line() {
        let text = "schema = \"1.0\"\ncolour = 1\n[sub]\non = true\nlater = 2\n\n[[rules]]\nname = \"a\"\n\
                    [[rules]]\nname = \"b\"\nenabeld = false\n";
        let found: Vec<(String, Option<usize>)> = unknown_toml_keys::<Root>(text)
            .into_iter()
            .map(|key| (key.path, key.line))
            .collect();
        assert_eq!(
            found,
            [
                ("colour".to_string(), Some(2)),
                ("sub.later".to_string(), Some(5)),
                ("rules[1].enabeld".to_string(), Some(11)),
            ]
        );
        let root: Root = toml::from_str(text).unwrap();
        assert_eq!(
            (root.schema.as_str(), root.rules[1].name.as_str()),
            ("1.0", "b")
        );
        assert!(
            root.sub.is_some_and(|sub| sub.on),
            "the known keys still read"
        );
        assert!(unknown_toml_keys::<Root>("schema = \"1.0\"\n").is_empty());
        assert!(
            unknown_toml_keys::<Root>("rules = 3").is_empty(),
            "a text that does not read"
        );
        let key = UnknownKey {
            path: "colour".into(),
            line: Some(2),
        };
        assert_eq!(key.to_string(), "line 2: colour");
    }

    #[test]
    fn a_json_field_the_record_written_back_lacks_is_named_by_its_path() {
        let found =
            json!({"seq": 2, "orders": [{"id": 1}, {"id": 2, "colour": "red"}], "note": {"a": 1}});
        let known = json!({"seq": 2, "orders": [{"id": 1}, {"id": 2}], "area": null});
        assert_eq!(
            unknown_json_fields(&found, &known),
            ["note", "orders[1].colour"]
        );
        assert!(unknown_json_fields(&known, &known).is_empty());
    }
}
