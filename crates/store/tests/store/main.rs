//! The store on a temporary data folder: the journal and its fold, `output/.partial/` settled by
//! the journal, the views, the lock and the inputs.

mod common;
mod index;
mod journal;
mod lock_and_files;
mod namespaces;
mod partial;
mod views;
