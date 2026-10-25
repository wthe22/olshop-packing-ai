//! The page logos: which images are candidates, their byte hash and the appearance
//! hash of a masked logo (courier.md items 1 and 3). The pictures are built in the test.

mod common;

use image::{DynamicImage, Rgba, RgbaImage};
use packing_pdf::{logo, read};
use pdfium_render::prelude::*;
use testkit::testdata;

/// The 3 x 2 DeviceRGB stream the in-band image carries, bytes 0..=17, uncompressed.
fn in_band_bytes() -> Vec<u8> {
    (0u8..18).collect()
}

/// The 5 x 4 DeviceRGB stream of the out-of-band image, bytes 60..=119.
fn out_of_band_bytes() -> Vec<u8> {
    (60u8..120).collect()
}

/// Build a small PDF out of ready-made object bodies (1-based): a proper xref, so PDFium reads it
/// as a normal file. The image streams are uncompressed, so `get_raw_image_data` gives back
/// exactly these bytes.
fn build_pdf(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut out: Vec<u8> = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (index, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in &offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// One 298 x 420 pt page: an image whose whole box is in the top 40 pt band (y 390…400) and one
/// far below it (y 100…120).
fn band_pdf() -> Vec<u8> {
    let contents = b"q 20 0 0 10 10 390 cm /Im0 Do Q\nq 30 0 0 20 10 100 cm /Im1 Do Q\n";
    let mut content = format!("<< /Length {} >>\nstream\n", contents.len()).into_bytes();
    content.extend_from_slice(contents);
    content.extend_from_slice(b"endstream");

    let image = |name: &str, width: usize, height: usize, bytes: &[u8]| {
        let mut body = format!(
            "<< /Type /XObject /Subtype /Image /Width {width} /Height {height} \
             /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>\nstream\n",
            bytes.len()
        )
        .into_bytes();
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\nendstream");
        let _ = name;
        body
    };

    build_pdf(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 298 420] \
          /Resources << /XObject << /Im0 5 0 R /Im1 6 0 R >> >> /Contents 4 0 R >>"
            .to_vec(),
        content,
        image("Im0", 3, 2, &in_band_bytes()),
        image("Im1", 5, 4, &out_of_band_bytes()),
    ])
}

#[test]
fn only_the_header_band_images_of_a_page_are_collected() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let path = common::temp_path("band-logos");
    std::fs::write(&path, band_pdf()).expect("write the probe pdf");
    let document = pdfium.load_pdf_from_file(&path, None).expect("load");
    let page = document.pages().get(0).expect("page");
    eprintln!("page height: {} pt", page.height().value);

    let logos = read::page_logos(&page, &document, &|_| false).expect("read the logos");
    // Only the image in the top band; the one at y 100…120 is not a candidate.
    assert_eq!(logos.len(), 1, "only the in-band image is a logo candidate");
    eprintln!("band logo hash: {}", logos[0].hash);

    // The byte hash is the XXH3-64 of the 18 stream bytes written above, a value computed outside
    // this crate, and the crate's own hash of the same bytes.
    assert_eq!(logos[0].hash, "f6912490d4c0eed5");
    assert_eq!(logos[0].hash, logo::hash_bytes(&in_band_bytes()));
    assert!(logos[0].appearance.is_some(), "the hash is unknown: decode");

    // A page whose logo bytes the table knows pays no decode.
    let known =
        read::page_logos(&page, &document, &|hash| hash == logos[0].hash).expect("read the logos");
    assert_eq!(known.len(), 1);
    assert_eq!(known[0].appearance, None);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_page_without_images_has_no_logos() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    // The fixtures carry text only.
    let path = testdata("labels-slip.pdf");
    let document = pdfium.load_pdf_from_file(&path, None).expect("load");
    let page = document.pages().get(0).expect("page");
    let logos = read::page_logos(&page, &document, &|_| false).expect("read the logos");
    assert!(logos.is_empty(), "a page without images has no logo");
}

/// The masked wordmark of the two rasters: a flat dark red base whose alpha channel is a
/// rectangle from 10% to 90% across and 20% to 80% down, sampled at the pixel centres.
fn masked_image(width: u32, height: u32) -> DynamicImage {
    let mut pixels = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let u = (f64::from(x) + 0.5) / f64::from(width);
            let v = (f64::from(y) + 0.5) / f64::from(height);
            let ink = (0.10..0.90).contains(&u) && (0.20..0.80).contains(&v);
            pixels.put_pixel(
                x,
                y,
                if ink {
                    Rgba([200, 16, 46, 255])
                } else {
                    Rgba([0, 0, 0, 0])
                },
            );
        }
    }
    DynamicImage::ImageRgba8(pixels)
}

#[test]
fn a_masked_logo_keeps_one_appearance_hash_across_a_re_raster() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let path = common::temp_path("masked-logos");
    let mut document = pdfium.create_new_pdf().expect("new pdf");
    for (width, height) in [(680u32, 156u32), (683, 157)] {
        let image = masked_image(width, height);
        let mut page = document
            .pages_mut()
            .create_page_at_end(PdfPagePaperSize::a4())
            .expect("page");
        page.objects_mut()
            .create_image_object(
                PdfPoints::new(10.0),
                PdfPoints::new(810.0),
                &image,
                Some(PdfPoints::new(100.0)),
                Some(PdfPoints::new(23.0)),
            )
            .expect("masked image object");
    }
    document.save_to_file(&path).expect("save");
    let reloaded = pdfium.load_pdf_from_file(&path, None).expect("reload");

    let mut byte_hashes = Vec::new();
    let mut appearance_hashes = Vec::new();
    for index in 0..2 {
        let page = reloaded.pages().get(index).expect("page");
        let logos = read::page_logos(&page, &reloaded, &|_| false).expect("read the logos");
        assert_eq!(logos.len(), 1, "the masked logo is a candidate");
        eprintln!(
            "page {}: bytes {} appearance {:?}",
            index + 1,
            logos[0].hash,
            logos[0].appearance
        );
        byte_hashes.push(logos[0].hash.clone());
        appearance_hashes.push(
            logos[0]
                .appearance
                .clone()
                .expect("an unknown byte hash decodes the image"),
        );
    }
    let _ = std::fs::remove_file(&path);

    // The encoded bytes of a masked logo name nothing and differ between the rasters...
    assert_ne!(byte_hashes[0], byte_hashes[1]);
    // ...while the appearance is one value for both (courier.md item 3).
    assert_eq!(appearance_hashes[0], appearance_hashes[1]);
    // The same picture through Pillow's `Image.LANCZOS` and this pipeline (unclamped f64 passes,
    // PIL's luminance, threshold 200, 512 packed bytes) gives the same value.
    assert_eq!(appearance_hashes[0], "6e160954c3421e15");
}

// --- the prompt picture: the logo as printed (the logo prompt) ---------------------------------

#[test]
fn the_prompt_picture_is_the_band_image_of_the_page() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let path = common::temp_path("picture");
    std::fs::write(&path, band_pdf()).expect("write the probe pdf");
    let document = pdfium.load_pdf_from_file(&path, None).expect("load");
    let page = document.pages().get(0).expect("page");

    // The in-band image the prompt asks about (3 x 2, the stream bytes 0..=17).
    let hash = logo::hash_bytes(&in_band_bytes());
    let png = read::page_picture(&page, &document, &hash)
        .expect("the picture")
        .expect("a page carries the image");
    let decoded = image::load_from_memory(&png).expect("decode").to_rgb8();
    // The image is under the cap: the picture keeps its own size.
    assert_eq!((decoded.width(), decoded.height()), (3, 2));

    // A hash the page does not carry gives no picture (the screen then shows the hash text)...
    assert!(
        read::page_picture(&page, &document, "ffffffffffffffff")
            .expect("the picture")
            .is_none()
    );
    // ...and the out-of-band image is never a candidate, its hash or not.
    assert!(
        read::page_picture(&page, &document, &logo::hash_bytes(&out_of_band_bytes()))
            .expect("the picture")
            .is_none()
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_masked_picture_is_the_wordmark_on_white_and_capped() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let path = common::temp_path("masked-picture");
    let mut document = pdfium.create_new_pdf().expect("new pdf");
    let image = masked_image(680, 156);
    let mut page = document
        .pages_mut()
        .create_page_at_end(PdfPagePaperSize::a4())
        .expect("page");
    page.objects_mut()
        .create_image_object(
            PdfPoints::new(10.0),
            PdfPoints::new(810.0),
            &image,
            Some(PdfPoints::new(100.0)),
            Some(PdfPoints::new(23.0)),
        )
        .expect("masked image object");
    document.save_to_file(&path).expect("save");
    let reloaded = pdfium.load_pdf_from_file(&path, None).expect("reload");
    let page = reloaded.pages().get(0).expect("page");
    let logos = read::page_logos(&page, &reloaded, &|_| false).expect("read the logos");
    assert_eq!(logos.len(), 1);

    let png = read::page_picture(&page, &reloaded, &logos[0].hash)
        .expect("the picture")
        .expect("the page carries the logo");
    let _ = std::fs::remove_file(&path);
    let decoded = image::load_from_memory(&png).expect("decode").to_rgb8();
    // The cap: 680 x 156 becomes 240 x 55.
    assert_eq!(decoded.width(), logo::MAX_PICTURE);
    assert_eq!(decoded.width().max(decoded.height()), logo::MAX_PICTURE);
    // Where the mask is 0 the pixel is white; the wordmark stays its dark red.
    let middle = decoded.height() / 2;
    assert_eq!(decoded.get_pixel(2, middle).0, [255, 255, 255]);
    let ink = decoded.get_pixel(decoded.width() / 2, middle).0;
    assert!(
        ink[0] > 150 && ink[1] < 80 && ink[2] < 80,
        "the wordmark is dark red, got {ink:?}"
    );
}

// --- reading a page must leave the document as it was ----------------------------------------

/// `get_processed_image` rewrites the image object's matrix in the page it is called on, and the
/// change survives a reload of the page: a second read of the same page lost a band image, and
/// the saved PDFs copy these pages. The appearance is therefore rendered on a scratch copy.
#[test]
fn reading_a_page_twice_gives_the_same_logos_and_keeps_the_document() {
    let Some(pdfium) = common::bind() else {
        eprintln!("skipped: run tools/get-pdfium.sh");
        return;
    };
    let path = common::temp_path("reread-logos");
    let mut document = pdfium.create_new_pdf().expect("new pdf");
    let mut page = document
        .pages_mut()
        .create_page_at_end(PdfPagePaperSize::a4())
        .expect("page");
    for (x, (width, height)) in [(10.0, (680u32, 156u32)), (150.0, (683, 157))] {
        page.objects_mut()
            .create_image_object(
                PdfPoints::new(x),
                PdfPoints::new(810.0),
                &masked_image(width, height),
                Some(PdfPoints::new(100.0)),
                Some(PdfPoints::new(23.0)),
            )
            .expect("masked image object");
    }
    drop(page);
    document.save_to_file(&path).expect("save");

    let file = read::open(&pdfium, &path).expect("open");
    let first = file
        .page_text(0, packing_engine::label::courier_data())
        .expect("first read")
        .logos;
    let second = file
        .page_text(0, packing_engine::label::courier_data())
        .expect("second read")
        .logos;
    let third = file
        .page_text(0, packing_engine::label::courier_data())
        .expect("third read")
        .logos;
    let _ = std::fs::remove_file(&path);

    assert_eq!(first.len(), 2, "both band images are candidates");
    assert!(first.iter().all(|logo| logo.appearance.is_some()));
    assert_eq!(first, second);
    assert_eq!(first, third);
}
