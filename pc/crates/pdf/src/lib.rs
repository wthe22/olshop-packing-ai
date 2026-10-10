//! PDF work for the packing app. This is the only crate that links PDFium; `packing-engine`
//! takes the text runs this crate produces and returns decisions.

use std::path::Path;

use pdfium_render::prelude::{Pdfium, PdfiumError};

pub mod packing_list;
pub mod read;
pub mod write;

/// Bind the pinned `pdfium.dll` found in `dll_dir`.
pub fn bind(dll_dir: &Path) -> Result<Pdfium, PdfiumError> {
    let bindings = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(dll_dir))?;
    Ok(Pdfium::new(bindings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfium_render::prelude::PdfPagePaperSize;

    #[test]
    fn binds_pdfium_and_adds_a_page() {
        let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
        if !vendor.join("pdfium.dll").exists() {
            eprintln!("skipped: run pc/tools/get-pdfium.sh");
            return;
        }
        let pdfium = bind(&vendor).expect("bind pdfium");
        let mut doc = pdfium.create_new_pdf().expect("create new pdf");
        doc.pages_mut()
            .create_page_at_end(PdfPagePaperSize::a4())
            .expect("add an A4 page");
        assert_eq!(doc.pages().len(), 1);
    }
}
