//! The committed fixtures are the writer's: writing them again gives the same bytes. A change to
//! the orders or the drawing is committed with `cargo run -p packing-fixtures`.

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn the_committed_fixtures_are_the_writers_output() {
    let dir = std::env::temp_dir().join(format!("packing-fixtures-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    packing_fixtures::write_all(&dir).expect("write the fixtures");
    let committed = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata");
    for file in packing_fixtures::FILES {
        let written = fs::read(dir.join(file)).expect("written file");
        let kept = fs::read(committed.join(file)).expect("committed file");
        assert!(
            written == kept,
            "testdata/{file} is not the writer's output: run `cargo run -p packing-fixtures`"
        );
    }
    let _ = fs::remove_dir_all(PathBuf::from(&dir));
}
