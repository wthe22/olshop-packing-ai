//! The page fingerprint (reading.md): a page named by what it prints, to tell whether the copies
//! of a repeated order are the same. It is not the verifier's page digest.

use xxhash_rust::xxh3::xxh3_64;

use crate::text::PageText;

/// XXH3-64 of the page's runs in page order — each run's text with its x and y rounded to 0.1 pt
/// — followed by the byte hashes of its header images.
pub fn page_fingerprint(page: &PageText) -> u64 {
    let mut bytes = Vec::new();
    for run in &page.runs {
        bytes.extend_from_slice(format!("{}\t{:.1}\t{:.1}\n", run.text, run.x, run.y).as_bytes());
    }
    for logo in &page.logos {
        bytes.extend_from_slice(logo.hash.as_bytes());
        bytes.push(b'\n');
    }
    xxh3_64(&bytes)
}
