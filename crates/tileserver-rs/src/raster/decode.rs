//! Decode PNG/JPEG/WebP bytes into a [`RasterImage`].
//!
//! This is the inverse of [`crate::raster::encode`] and is used by
//! the mosaic path when an asset is cached as compressed bytes (e.g.,
//! from a PMTiles source) and needs to enter the array pipeline for
//! band math or pixel selection.

use image::ImageFormat;

use super::RasterImage;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("image decoding failed: {0}")]
    Image(#[from] image::ImageError),
}

/// Decode any PNG/JPEG/WebP/etc byte buffer (format auto-detected) into
/// a 4-band [`RasterImage`] (RGBA).
///
/// # Errors
/// Returns [`DecodeError`] on unknown format or corrupt payload.
pub fn from_bytes(buf: &[u8]) -> Result<RasterImage, DecodeError> {
    let img = image::load_from_memory(buf)?.into_rgba8();
    Ok(RasterImage::from(&img))
}

/// Decode a byte buffer with an explicit format hint. Slightly faster
/// than [`from_bytes`] because the decoder isn't guessing.
///
/// # Errors
/// Returns [`DecodeError`] on mismatched format or corrupt payload.
pub fn from_bytes_format(buf: &[u8], format: ImageFormat) -> Result<RasterImage, DecodeError> {
    let img = image::load_from_memory_with_format(buf, format)?.into_rgba8();
    Ok(RasterImage::from(&img))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::encode;
    use ndarray::array;

    #[test]
    fn round_trip_png() {
        let data = array![[[100.0_f32, 150.0], [200.0, 250.0]]];
        let original = RasterImage::from_opaque(data, None);
        let png = encode::to_png(&original).unwrap();
        let decoded = from_bytes(&png).unwrap();
        assert_eq!(decoded.band_count(), 4);
        assert_eq!(decoded.width(), 2);
        assert_eq!(decoded.height(), 2);
        let d = decoded.data();
        assert!((d[[0, 0, 0]] - 100.0).abs() < f32::EPSILON);
        assert!((d[[0, 1, 1]] - 250.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_garbage_input() {
        let err = from_bytes(b"not an image").unwrap_err();
        assert!(matches!(err, DecodeError::Image(_)));
    }
}
