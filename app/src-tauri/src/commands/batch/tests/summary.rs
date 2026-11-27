//! The Summary screen's summaries, read from the saved date without writing a file.

use std::fs;

use packing_engine::ids::BatchId;
use packing_pdf::summary_view::Part;

use super::{DAY, read, save, skipped, started};
use crate::commands::day::summary::summary_of;

#[test]
fn a_summary_is_read_without_writing_a_file() {
    let Some((program, state, _guard)) = started("app-summary") else {
        return skipped();
    };
    read(&state);
    save(&state);
    let output = program.path().join("data").join(DAY);
    fs::remove_file(output.join("1 Summary.pdf")).unwrap();

    let batch = summary_of(&state, DAY, Some(BatchId::new(1)), None).unwrap();
    assert!(batch.counts.starts_with("4 orders"), "{}", batch.counts);
    assert!(matches!(batch.parts.last(), Some(Part::Picks { .. })));
    let mine = summary_of(&state, DAY, Some(BatchId::new(1)), Some("Toko Lain")).unwrap();
    assert!(mine.title.contains("Toko Lain"), "{}", mine.title);
    let day = summary_of(&state, DAY, None, None).unwrap();
    assert!(day.counts.contains("1 batch"), "{}", day.counts);

    assert!(!output.join("1 Summary.pdf").exists());
    assert!(!output.join("Day Summary.pdf").exists());
    assert!(summary_of(&state, DAY, Some(BatchId::new(9)), None).is_err());
    assert!(summary_of(&state, DAY, None, Some("No such account")).is_err());
}
