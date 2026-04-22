//! Lossless bridges between [`RasterImage`] and `image::RgbaImage`.
//!
//! These conversions exist so the new ndarray-backed pipeline can
//! interoperate with the legacy `image::RgbaImage` used by the existing
//! COG + STAC mosaic code during the migration. After all raster callers
//! have moved to `RasterImage`, the conversion from `RgbaImage` will be
//! retired; the conversion to `RgbaImage` remains because `mbgl-sys`
//! (MapLibre Native renderer) consumes RGBA bytes.

use image::{Rgba, RgbaImage};
use ndarray::{Array2, Array3};

use super::RasterImage;

impl From<&RgbaImage> for RasterImage {
    /// Convert an `image::RgbaImage` (u8 packed RGBA) into a 4-band
    /// [`RasterImage`] where channel 0 is R, 1 is G, 2 is B, 3 is A.
    ///
    /// The mask is derived from the alpha channel: `alpha == 0 ⇒ masked`.
    /// All u8 values are converted to `f32` in `[0.0, 255.0]` (not
    /// normalised to `[0.0, 1.0]` — mosaic methods compare absolute
    /// pixel values, not normalised ones, matching rio-tiler semantics).
    fn from(img: &RgbaImage) -> Self {
        let (w, h) = img.dimensions();
        let (w, h) = (w as usize, h as usize);
        let mut data = Array3::<f32>::zeros((4, h, w));
        let mut mask = Array2::from_elem((h, w), false);
        for (x, y, &Rgba([r, g, b, a])) in img.enumerate_pixels() {
            let (xu, yu) = (x as usize, y as usize);
            data[[0, yu, xu]] = f32::from(r);
            data[[1, yu, xu]] = f32::from(g);
            data[[2, yu, xu]] = f32::from(b);
            data[[3, yu, xu]] = f32::from(a);
            if a == 0 {
                mask[[yu, xu]] = true;
            }
        }
        RasterImage::new(data, mask, None)
    }
}

/// Convert a [`RasterImage`] to an `image::RgbaImage`.
///
/// Band selection rules (match common titiler behaviour):
/// - 1 band: greyscale — R=G=B=band0, A=255 (or 0 where masked).
/// - 2 bands: greyscale + alpha — R=G=B=band0, A=band1.
/// - 3 bands: RGB — R=band0, G=band1, B=band2, A=255 (or 0 where masked).
/// - 4+ bands: RGBA — R=band0, G=band1, B=band2, A=band3.
///
/// Values are clamped to `[0, 255]` and cast to `u8`.
///
/// # Errors
///
/// Returns an error if the image has zero bands (should be unreachable
/// given [`RasterImage`] invariants but validated defensively).
pub fn rgba_from_raster(img: &RasterImage) -> Result<RgbaImage, String> {
    let (bands, h, w) = img.data().dim();
    if bands == 0 {
        return Err("RasterImage has 0 bands".into());
    }
    let mut out = RgbaImage::new(w as u32, h as u32);
    let data = img.data();
    let mask = img.mask();
    for y in 0..h {
        for x in 0..w {
            let masked = mask[[y, x]];
            let (r, g, b, a) = pack_rgba(data, bands, y, x, masked);
            out.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
        }
    }
    Ok(out)
}

fn pack_rgba(
    data: &Array3<f32>,
    bands: usize,
    y: usize,
    x: usize,
    masked: bool,
) -> (u8, u8, u8, u8) {
    let clamp = |v: f32| v.clamp(0.0, 255.0) as u8;
    match bands {
        1 => {
            let v = clamp(data[[0, y, x]]);
            (v, v, v, if masked { 0 } else { 255 })
        }
        2 => {
            let v = clamp(data[[0, y, x]]);
            let a = clamp(data[[1, y, x]]);
            (v, v, v, if masked { 0 } else { a })
        }
        3 => {
            let r = clamp(data[[0, y, x]]);
            let g = clamp(data[[1, y, x]]);
            let b = clamp(data[[2, y, x]]);
            (r, g, b, if masked { 0 } else { 255 })
        }
        _ => {
            let r = clamp(data[[0, y, x]]);
            let g = clamp(data[[1, y, x]]);
            let b = clamp(data[[2, y, x]]);
            let a = clamp(data[[3, y, x]]);
            (r, g, b, if masked { 0 } else { a })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_to_raster_round_trip_opaque() {
        let mut src = RgbaImage::new(2, 2);
        src.put_pixel(0, 0, Rgba([10, 20, 30, 255]));
        src.put_pixel(1, 0, Rgba([40, 50, 60, 255]));
        src.put_pixel(0, 1, Rgba([70, 80, 90, 255]));
        src.put_pixel(1, 1, Rgba([100, 110, 120, 255]));

        let raster = RasterImage::from(&src);
        assert_eq!(raster.band_count(), 4);
        assert!(raster.is_fully_opaque());

        let round = rgba_from_raster(&raster).unwrap();
        assert_eq!(round.dimensions(), src.dimensions());
        assert_eq!(round.get_pixel(0, 0), &Rgba([10, 20, 30, 255]));
        assert_eq!(round.get_pixel(1, 1), &Rgba([100, 110, 120, 255]));
    }

    #[test]
    fn rgba_to_raster_mask_from_zero_alpha() {
        let mut src = RgbaImage::new(2, 1);
        src.put_pixel(0, 0, Rgba([10, 20, 30, 0]));
        src.put_pixel(1, 0, Rgba([40, 50, 60, 255]));
        let raster = RasterImage::from(&src);
        assert!(raster.mask()[[0, 0]], "alpha=0 pixel must be masked");
        assert!(!raster.mask()[[0, 1]], "alpha=255 pixel must not be masked");
    }

    #[test]
    fn raster_to_rgba_single_band_greyscale() {
        let data = ndarray::array![[[128.0_f32, 200.0], [50.0, 255.0]]];
        let raster = RasterImage::from_opaque(data, None);
        let out = rgba_from_raster(&raster).unwrap();
        assert_eq!(out.get_pixel(0, 0), &Rgba([128, 128, 128, 255]));
        assert_eq!(out.get_pixel(1, 1), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn raster_to_rgba_masked_pixel_alpha_zero() {
        let data = ndarray::array![[[200.0_f32]]];
        let mask = Array2::from_elem((1, 1), true);
        let raster = RasterImage::new(data, mask, None);
        let out = rgba_from_raster(&raster).unwrap();
        assert_eq!(out.get_pixel(0, 0), &Rgba([200, 200, 200, 0]));
    }

    #[test]
    fn raster_to_rgba_clamps_out_of_range() {
        let data = ndarray::array![[[300.0_f32, -50.0], [127.5, 0.0]]];
        let raster = RasterImage::from_opaque(data, None);
        let out = rgba_from_raster(&raster).unwrap();
        assert_eq!(out.get_pixel(0, 0), &Rgba([255, 255, 255, 255]));
        assert_eq!(out.get_pixel(1, 0), &Rgba([0, 0, 0, 255]));
        assert_eq!(out.get_pixel(0, 1), &Rgba([127, 127, 127, 255]));
    }
}
