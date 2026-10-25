//! The courier-logo fingerprints of a page ([courier deduction]).
//!
//! A label's logo is an embedded image. Two hashes are taken of it, both **XXH3-64** (seed 0) of
//! 16 lowercase hex characters:
//!
//! - [`hash_bytes`] — the image's **encoded bytes** (`FPDFImageObj_GetImageDataRaw`, *not* the
//!   decoded pixels). One stable value per courier asset; the first clue (courier.md item 1).
//! - [`appearance_hash`] — the image's **appearance**, for a *masked* logo whose encoded bytes
//!   name nothing and which the platform re-rastered between exports (J&T Cargo; item 3). It is
//!   computed only where the byte hash names nothing, so a page whose logo
//!   the table knows pays no decode.
//!
//! # The appearance hash, exactly
//!
//! The image is composited onto white with its `/SMask` as the alpha channel, then scaled into a
//! fixed 64 × 64 white canvas, thresholded at 200/255, and the 1-bit canvas is hashed:
//!
//! 1. **Composite** — PDFium's rendered bitmap is straight (not premultiplied) RGBA; a pixel
//!    becomes `round((channel * alpha + 255 * (255 - alpha)) / 255)`, so a fully transparent
//!    pixel is white.
//! 2. **Fit** — `scale = min(64 / width, 64 / height)`; the target size is
//!    `(round_ties_even(width * scale), round_ties_even(height * scale))`, at least 1 × 1.
//! 3. **Resize** — Pillow's `Image.LANCZOS`: a separable Lanczos-windowed sinc, filter support
//!    3, whose coefficients are computed on the output pixel centres
//!    (`center = (i + 0.5) * in / out`, weights `lanczos((j + 0.5 - center) * out / in)`, the
//!    support widened by `in / out` and the weights normalised). The coefficients and the two
//!    passes are `f64`; only the final pixel is rounded and clamped to 0…255. Measured: the
//!    thresholded hashes match Pillow's `Image.LANCZOS` on the probe shapes, so a value taken
//!    with Pillow can be checked against this.
//! 4. **Paste** — centred on a 64 × 64 white canvas at `((64 - tw) / 2, (64 - th) / 2)`.
//! 5. **Threshold and pack** — a pixel is **ink** (bit 1) when its luminance is **under 200**,
//!    else bit 0 (so the dark wordmark is the 1s). The luminance is PIL's `convert("L")`:
//!    `(r * 19595 + g * 38470 + b * 7471 + 32768) >> 16`. The canvas is packed **8 pixels per
//!    byte, most significant bit first** — PIL's mode `"1"` layout — so a 64 × 64 canvas is
//!    512 bytes, and those bytes are hashed.
//!
//! [courier deduction]: ../../../docs/software-design/labels/courier.md

use xxhash_rust::xxh3::xxh3_64;

/// The canvas the appearance hash is taken on (courier.md item 3).
const CANVAS: usize = 64;

/// A pixel under this luminance is ink (courier.md item 3: the threshold is 200/255).
const THRESHOLD: u32 = 200;

/// XXH3-64 (seed 0) of image bytes, as 16 lowercase hex characters.
pub fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:016x}", xxh3_64(bytes))
}

/// The longest side of a prompt picture, in pixels (the logo prompt shows the logo as printed,
/// small enough for the dialog).
pub const MAX_PICTURE: u32 = 240;

/// The image composited onto white and written as **PNG bytes**, its longest side capped at
/// [`MAX_PICTURE`] px. The logo prompt reads a logo back from the PDF this way; a masked logo
/// becomes its wordmark on white (the `/SMask` is the alpha, step 1 of the module comment).
/// `None` when the pixels cannot be built or encoded.
pub fn picture_png(rgba: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    assert_eq!(
        rgba.len(),
        (width as usize) * (height as usize) * 4,
        "the picture needs width * height RGBA pixels"
    );
    let rgb = composite_on_white(rgba, width as usize, height as usize);
    let mut image = image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(width, height, rgb)?);
    if image.width().max(image.height()) > MAX_PICTURE {
        image = image.resize(
            MAX_PICTURE,
            MAX_PICTURE,
            image::imageops::FilterType::Lanczos3,
        );
    }
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .ok()?;
    Some(bytes)
}

/// XXH3-64 of an image's appearance: `rgba` is the image composited onto white, `width` x
/// `height` pixels of straight RGBA. See the module comment for the exact steps.
pub fn appearance_hash(rgba: &[u8], width: usize, height: usize) -> String {
    assert_eq!(
        rgba.len(),
        width * height * 4,
        "the appearance hash needs width * height RGBA pixels"
    );
    let rgb = composite_on_white(rgba, width, height);
    let scale = (CANVAS as f64 / width as f64).min(CANVAS as f64 / height as f64);
    let tw = ((width as f64 * scale).round_ties_even() as usize).max(1);
    let th = ((height as f64 * scale).round_ties_even() as usize).max(1);
    resample(&rgb, width, height, tw, th)
}

/// The image composited onto white with its alpha channel; a fully transparent pixel is white.
fn composite_on_white(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgb = vec![255u8; width * height * 3];
    for pixel in 0..width * height {
        let alpha = u32::from(rgba[pixel * 4 + 3]);
        for channel in 0..3 {
            let value = u32::from(rgba[pixel * 4 + channel]);
            rgb[pixel * 3 + channel] = ((value * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
        }
    }
    rgb
}

/// Scale the image to `tw` x `th` on a white 64 x 64 canvas, threshold it and hash the packed
/// 1-bit canvas (steps 3-5 of the module comment).
fn resample(rgb: &[u8], width: usize, height: usize, tw: usize, th: usize) -> String {
    let source: Vec<f64> = rgb.iter().map(|value| f64::from(*value)).collect();
    let horizontal = resample_axis(&source, width, height, tw, true);
    let small = resample_axis(&horizontal, height, tw, th, false);
    let left = (CANVAS - tw) / 2;
    let top = (CANVAS - th) / 2;
    let mut packed = vec![0u8; CANVAS * (CANVAS / 8)];
    for y in 0..th {
        for x in 0..tw {
            let pixel = (y * tw + x) * 3;
            // Only the output pixel is clamped and rounded; an intermediate result is not, which
            // is what makes the two rasters of one wordmark agree.
            let luminance = luminance(
                clamp_pixel(small[pixel]),
                clamp_pixel(small[pixel + 1]),
                clamp_pixel(small[pixel + 2]),
            );
            if luminance < THRESHOLD {
                let (x, y) = (left + x, top + y);
                packed[y * (CANVAS / 8) + x / 8] |= 1 << (7 - (x % 8));
            }
        }
    }
    hash_bytes(&packed)
}

/// PIL's `convert("L")`: ITU-R 601-2 luma in the same fixed point arithmetic.
fn luminance(r: u8, g: u8, b: u8) -> u32 {
    (u32::from(r) * 19595 + u32::from(g) * 38470 + u32::from(b) * 7471 + 32768) >> 16
}

/// One pass of the resize: `source` holds `cross` rows (horizontal) or columns (vertical) of RGB
/// pixels, `in_size` per line, and the result has `out_size` per line. The values stay `f64`.
fn resample_axis(
    source: &[f64],
    in_size: usize,
    cross: usize,
    out_size: usize,
    horizontal: bool,
) -> Vec<f64> {
    let table = coefficients(in_size, out_size);
    let mut out = vec![0.0f64; out_size * cross * 3];
    for line in 0..cross {
        for (i, (first, weights)) in table.iter().enumerate() {
            let first = *first;
            let mut sums = [0.0f64; 3];
            for (offset, weight) in weights.iter().enumerate() {
                let pixel = if horizontal {
                    (line * in_size + first + offset) * 3
                } else {
                    ((first + offset) * cross + line) * 3
                };
                for channel in 0..3 {
                    sums[channel] += source[pixel + channel] * weight;
                }
            }
            let pixel = if horizontal {
                (line * out_size + i) * 3
            } else {
                (i * cross + line) * 3
            };
            out[pixel..pixel + 3].copy_from_slice(&sums);
        }
    }
    out
}

fn clamp_pixel(value: f64) -> u8 {
    if value <= 0.0 {
        0
    } else if value >= 255.0 {
        255
    } else {
        (value + 0.5).floor() as u8
    }
}

/// Pillow's Lanczos coefficient table for one axis: the split of each output pixel's window over
/// the input lines, normalised.
fn coefficients(in_size: usize, out_size: usize) -> Vec<(usize, Vec<f64>)> {
    let scale = in_size as f64 / out_size as f64;
    let filter_scale = scale.max(1.0);
    let support = 3.0 * filter_scale;
    let inverse = 1.0 / filter_scale;
    (0..out_size)
        .map(|i| {
            let center = (i as f64 + 0.5) * scale;
            let low = ((center - support + 0.5).floor() as isize).max(0) as usize;
            let high =
                ((center + support + 0.5).floor() as isize).clamp(0, in_size as isize) as usize;
            let (low, high) = if low >= high {
                (in_size.saturating_sub(1), in_size.min(1))
            } else {
                (low, high)
            };
            let mut weights: Vec<f64> = (low..high)
                .map(|j| lanczos((j as f64 + 0.5 - center) * inverse))
                .collect();
            let total: f64 = weights.iter().sum();
            if total != 0.0 {
                for weight in &mut weights {
                    *weight /= total;
                }
            }
            (low, weights)
        })
        .collect()
}

/// The Lanczos kernel, Pillow's `LANCZOS` filter: `sinc(x) * sinc(x / 3)`.
fn lanczos(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    if x.abs() >= 3.0 {
        return 0.0;
    }
    let scaled = std::f64::consts::PI * x;
    scaled.sin() * (scaled / 3.0).sin() * 3.0 / (scaled * scaled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_xxh3_value_of_a_short_buffer() {
        // The published XXH3-64 of the empty input.
        assert_eq!(hash_bytes(&[]), "2d06800538d394c2");
    }

    #[test]
    fn a_white_image_has_no_ink_and_a_dark_one_is_all_ink() {
        let white = vec![255u8; 4 * 4 * 4];
        let dark = [0u8, 0, 0, 255].repeat(16);
        let white_hash = appearance_hash(&white, 4, 4);
        assert_eq!(white_hash.len(), 16);
        // A fully white canvas packs to 512 zero bytes.
        assert_eq!(white_hash, hash_bytes(&[0u8; 512]));
        // A fully opaque black image is ink everywhere: 512 bytes of 0xFF.
        assert_eq!(appearance_hash(&dark, 4, 4), hash_bytes(&[0xFFu8; 512]));
    }

    #[test]
    fn a_picture_is_the_image_on_white_smaller_than_the_cap() {
        // Two opaque pixels: dark red, black. Transparent pixels become white.
        let mut rgba = vec![0u8; 3 * 2 * 4];
        rgba[0..4].copy_from_slice(&[200, 16, 46, 255]);
        rgba[4..8].copy_from_slice(&[0, 0, 0, 0]);
        let png = picture_png(&rgba, 3, 2).expect("png");
        let decoded = image::load_from_memory(&png).expect("decode").to_rgb8();
        assert_eq!((decoded.width(), decoded.height()), (3, 2));
        assert_eq!(decoded.get_pixel(0, 0).0, [200, 16, 46]);
        // Where the alpha is 0 the pixel is white, not black.
        assert_eq!(decoded.get_pixel(1, 0).0, [255, 255, 255]);
        assert_eq!(decoded.get_pixel(2, 1).0, [255, 255, 255]);
    }

    #[test]
    fn a_picture_is_capped_to_the_longest_side() {
        // 480 x 120 of one colour: the cap keeps the ratio, longest side 240.
        let rgba = [0u8, 0, 0, 255].repeat(480 * 120);
        let png = picture_png(&rgba, 480, 120).expect("png");
        let decoded = image::load_from_memory(&png).expect("decode").to_rgb8();
        assert_eq!(decoded.width().max(decoded.height()), MAX_PICTURE);
        assert_eq!((decoded.width(), decoded.height()), (240, 60));
    }
}
