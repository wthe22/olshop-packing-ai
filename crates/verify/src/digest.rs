//! The page digest (verification.md): XXH3-64 of a page as drawn, in a fixed form — its boxes and
//! rotation, its decoded content, and its resources with every reference resolved — so it ignores
//! how a writer stores the page and keeps what it draws. An object shared by pages is digested once
//! per file.

use std::collections::{HashMap, HashSet};

use lopdf::{Dictionary, Document, Object, ObjectId};
use xxhash_rust::xxh3::xxh3_64;

/// Keys that say how an object is stored, not what it draws.
const STORAGE_KEYS: [&[u8]; 4] = [b"Length", b"Filter", b"DecodeParms", b"Parent"];

pub(crate) struct Digester<'a> {
    doc: &'a Document,
    done: HashMap<ObjectId, u64>,
    open: HashSet<ObjectId>,
}

impl<'a> Digester<'a> {
    pub fn new(doc: &'a Document) -> Self {
        Self {
            doc,
            done: HashMap::new(),
            open: HashSet::new(),
        }
    }

    /// The digests of the document's pages, in page order.
    pub fn pages(&mut self) -> Vec<u64> {
        let pages: Vec<ObjectId> = self.doc.get_pages().into_values().collect();
        pages.into_iter().map(|page| self.page(page)).collect()
    }

    fn page(&mut self, page: ObjectId) -> u64 {
        let mut form = Vec::new();
        let media = self.inherited(page, b"MediaBox");
        for (key, value) in [
            (&b"MediaBox"[..], media.clone()),
            (b"CropBox", self.inherited(page, b"CropBox").or(media)),
            (b"Rotate", self.inherited(page, b"Rotate")),
        ] {
            form.extend_from_slice(key);
            match value {
                Some(value) => self.write(&value, &mut form),
                None => form.push(b'n'),
            }
        }
        form.extend_from_slice(b"content");
        let content = self.doc.get_page_content(page);
        write_bytes(&content, &mut form);
        form.extend_from_slice(b"resources");
        match self.inherited(page, b"Resources") {
            Some(resources) => self.write(&resources, &mut form),
            None => form.push(b'n'),
        }
        xxh3_64(&form)
    }

    /// A page attribute, from the page or the nearest node of the page tree above it.
    fn inherited(&self, page: ObjectId, key: &[u8]) -> Option<Object> {
        let mut node = self.doc.get_dictionary(page).ok()?;
        for _ in 0..64 {
            if let Ok(value) = node.get(key) {
                return Some(value.clone());
            }
            let parent = node.get(b"Parent").and_then(Object::as_reference).ok()?;
            node = self.doc.get_dictionary(parent).ok()?;
        }
        None
    }

    fn write(&mut self, object: &Object, form: &mut Vec<u8>) {
        match object {
            Object::Null => form.push(b'n'),
            Object::Boolean(value) => form.push(if *value { b't' } else { b'f' }),
            // A number is its value however it is spelled: PDFium writes `1.0` as `1`.
            Object::Integer(value) => write_number(*value as f64, form),
            Object::Real(value) => write_number(*value as f64, form),
            Object::Name(name) => {
                form.push(b'/');
                write_bytes(name, form);
            }
            Object::String(bytes, _) => {
                form.push(b'(');
                write_bytes(bytes, form);
            }
            Object::Array(items) => {
                form.push(b'[');
                for item in items {
                    self.write(item, form);
                }
                form.push(b']');
            }
            Object::Dictionary(dict) => self.write_dictionary(dict, form),
            Object::Stream(stream) => {
                self.write_dictionary(&stream.dict, form);
                match stream.decompressed_content() {
                    Ok(content) => write_bytes(&content, form),
                    Err(_) => {
                        // A filter lopdf cannot decode (an image's DCT): the stored bytes and their
                        // filter are what is drawn.
                        if let Ok(filter) = stream.dict.get(b"Filter") {
                            self.write(filter, form);
                        }
                        write_bytes(&stream.content, form);
                    }
                }
            }
            Object::Reference(id) => {
                form.push(b'R');
                match self.referenced(*id) {
                    Some(digest) => form.extend_from_slice(&digest.to_le_bytes()),
                    None => form.push(b'^'),
                }
            }
        }
    }

    fn write_dictionary(&mut self, dict: &Dictionary, form: &mut Vec<u8>) {
        let mut keys: Vec<&Vec<u8>> = dict
            .iter()
            .map(|(key, _)| key)
            .filter(|key| !STORAGE_KEYS.contains(&key.as_slice()))
            .collect();
        keys.sort();
        form.push(b'<');
        for key in keys {
            write_bytes(key, form);
            let value = dict.get(key).expect("a key of this dictionary");
            self.write(value, form);
        }
        form.push(b'>');
    }

    /// The digest of a referenced object, once per object; `None` for one being written (a loop).
    fn referenced(&mut self, id: ObjectId) -> Option<u64> {
        if let Some(digest) = self.done.get(&id) {
            return Some(*digest);
        }
        if !self.open.insert(id) {
            return None;
        }
        let mut form = Vec::new();
        match self.doc.get_object(id) {
            Ok(object) => self.write(object, &mut form),
            Err(_) => form.push(b'?'),
        }
        self.open.remove(&id);
        let digest = xxh3_64(&form);
        self.done.insert(id, digest);
        Some(digest)
    }
}

/// Bytes with their length in front, so no two sequences write the same form.
fn write_bytes(bytes: &[u8], form: &mut Vec<u8>) {
    form.extend_from_slice(format!("{}:", bytes.len()).as_bytes());
    form.extend_from_slice(bytes);
}

/// A number to four decimals, so an integer and a real of one value write the same.
fn write_number(value: f64, form: &mut Vec<u8>) {
    form.extend_from_slice(format!("n{value:.4};").as_bytes());
}
