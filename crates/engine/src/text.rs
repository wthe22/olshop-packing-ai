//! A page's text runs with x, y (the input from `packing-pdf`).

/// One piece of text on a page: `x` is the left edge, `y` the baseline, in PDF points with
/// `y` growing upwards.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub x: f64,
    pub y: f64,
    pub text: String,
}

/// One candidate courier logo of a page: an image whose whole box lies in the page's top band.
/// `packing-pdf` builds these; the courier rule looks the hashes up in
/// `couriers.toml` (courier.md items 1 and 3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogoImage {
    /// XXH3-64 (16 lowercase hex) of the image's **encoded bytes**
    /// (`FPDFImageObj_GetImageDataRaw`).
    pub hash: String,
    /// XXH3-64 (16 lowercase hex) of the image's **appearance**, computed by `packing-pdf` only
    /// when `hash` is not yet known to `couriers.toml` (`logo`/`not_courier`). `None` means the
    /// table already knew the bytes, so nothing had to be decoded.
    pub appearance: Option<String>,
}

/// What `packing-pdf` reads from one page; the engine's only input from a PDF.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PageText {
    /// The page's whole text in reading order (PDFium `PdfPageText::all`), for the Order ID,
    /// tracking ID, `In transit by` and courier searches.
    pub text: String,
    /// The runs rebuilt from the characters, for the slip table.
    pub runs: Vec<Run>,
    /// Page width in points.
    pub width: f64,
    /// The images whose whole box lies in the page's top band, in page order.
    pub logos: Vec<LogoImage>,
}
