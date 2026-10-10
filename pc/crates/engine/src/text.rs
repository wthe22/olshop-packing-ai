//! A page's text runs with x, y (the input from `packing-pdf`).

/// One piece of text on a page: `x` is the left edge, `y` the baseline, in PDF points with
/// `y` growing upwards (as pypdf gave them to the Python script).
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub x: f64,
    pub y: f64,
    pub text: String,
}

/// What `packing-pdf` reads from one page; the engine's only input from a PDF.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PageText {
    /// The page's whole text in reading order (PDFium `PdfPageText::all`), for the Order ID,
    /// tracking ID, `In transit by` and courier searches.
    pub text: String,
    /// The runs rebuilt from the characters (07 › *Text runs from PDFium*), for the slip table.
    pub runs: Vec<Run>,
    /// Page width in points.
    pub width: f64,
}
