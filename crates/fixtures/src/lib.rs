//! The made-up label fixtures of `testdata/` and their answers. The pages are drawn the way the
//! downloads are made — one character per text object — and each answer is written from the same
//! orders, never by reading the pages: this crate uses no other crate of the workspace and no
//! PDFium.

mod answer;
mod draw;
mod metrics;
mod orders;
mod pdf;

use std::fs;
use std::path::Path;

/// The files this crate writes, in the order it writes them.
pub const FILES: [&str; 6] = [
    "labels-slip.pdf",
    "labels-plain.pdf",
    "expected-labels.json",
    "labels-repeat-1.pdf",
    "labels-repeat-2.pdf",
    "expected-repeat.json",
];

/// Write the fixtures and their answers into `dir`.
pub fn write_all(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let slip = orders::slip_labels();
    let plain = orders::plain_labels();
    pdf::write(&dir.join(FILES[0]), &pages(&slip))?;
    pdf::write(&dir.join(FILES[1]), &pages(&plain))?;
    let all: Vec<&orders::Label> = slip.iter().chain(&plain).collect();
    fs::write(dir.join(FILES[2]), answer::expected_labels(&all))?;

    let repeat = orders::repeat_inputs();
    pdf::write(&dir.join(FILES[3]), &pages(&repeat[0]))?;
    pdf::write(&dir.join(FILES[4]), &pages(&repeat[1]))?;
    fs::write(
        dir.join(FILES[5]),
        answer::expected_repeat(&repeat, &FILES[3..5]),
    )
}

fn pages(labels: &[orders::Label]) -> Vec<draw::Page> {
    labels.iter().flat_map(draw::label_pages).collect()
}
