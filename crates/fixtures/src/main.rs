//! `cargo run -p packing-fixtures [dir]`: write the label fixtures and their answer into `dir`
//! (default: the repository's `testdata/`).

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata"));
    match packing_fixtures::write_all(&dir) {
        Ok(()) => {
            for file in packing_fixtures::FILES {
                println!("{}", dir.join(file).display());
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("cannot write the fixtures into {}: {error}", dir.display());
            ExitCode::FAILURE
        }
    }
}
