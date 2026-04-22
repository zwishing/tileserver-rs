//! The [`RasterImage`] type — an ndarray-backed multi-band raster.
//!
//! `RasterImage` carries the pixel data as `Array3<f32>` with shape
//! `(bands, height, width)`, plus a boolean mask, nodata value, and
//! spatial metadata. All mosaic + band-math + colormap operations return
//! new `RasterImage`s; PNG/JPEG encoding is deferred to the HTTP
//! response boundary (see [`crate::raster::encode`]).
//!
//! # Invariants
//!
//! - `data.shape() == [bands, height, width]` (always 3 dimensions).
//! - `mask.shape() == [height, width]` (single plane; a pixel is either
//!   valid in all bands or masked in all bands).
//! - `bands >= 1`, `height >= 1`, `width >= 1`.
//! - `f32` is canonical; inputs in `u8`/`u16`/`i16`/`f64` convert on
//!   construction and back-convert on encode.

use ndarray::{Array2, Array3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RasterMetadata {
    pub bands: usize,
    pub height: usize,
    pub width: usize,
    pub nodata: Option<f64>,
    pub bounds: Option<[f64; 4]>,
    pub crs_epsg: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct RasterImage {
    data: Array3<f32>,
    mask: Array2<bool>,
    nodata: Option<f64>,
    bounds: Option<[f64; 4]>,
    crs_epsg: Option<u32>,
}

impl RasterImage {
    /// Construct a new `RasterImage` from raw pixel data and a validity mask.
    ///
    /// # Panics
    ///
    /// Panics if `data.shape()` is not 3D, or if `mask.shape()` does not
    /// match `data`'s `(height, width)` plane. This is a programmer error
    /// — caller must ensure shape consistency before constructing.
    #[must_use]
    pub fn new(data: Array3<f32>, mask: Array2<bool>, nodata: Option<f64>) -> Self {
        let (_, h, w) = data.dim();
        assert_eq!(
            mask.dim(),
            (h, w),
            "mask dimensions ({mh}, {mw}) must match data HxW ({h}, {w})",
            mh = mask.dim().0,
            mw = mask.dim().1
        );
        assert!(h >= 1 && w >= 1, "raster dimensions must be >= 1");
        assert!(data.dim().0 >= 1, "band count must be >= 1");
        Self {
            data,
            mask,
            nodata,
            bounds: None,
            crs_epsg: None,
        }
    }

    /// Construct a fully-valid `RasterImage` (no pixels masked).
    #[must_use]
    pub fn from_opaque(data: Array3<f32>, nodata: Option<f64>) -> Self {
        let (_, h, w) = data.dim();
        let mask = Array2::from_elem((h, w), false);
        Self::new(data, mask, nodata)
    }

    /// Attach a WGS-84 bounding box `[west, south, east, north]` for
    /// reprojection + tile alignment sanity checks.
    #[must_use]
    pub fn with_bounds(mut self, bounds: [f64; 4]) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn with_crs(mut self, epsg: u32) -> Self {
        self.crs_epsg = Some(epsg);
        self
    }

    #[must_use]
    pub fn metadata(&self) -> RasterMetadata {
        let (bands, height, width) = self.data.dim();
        RasterMetadata {
            bands,
            height,
            width,
            nodata: self.nodata,
            bounds: self.bounds,
            crs_epsg: self.crs_epsg,
        }
    }

    #[must_use]
    pub fn data(&self) -> &Array3<f32> {
        &self.data
    }

    #[must_use]
    pub fn mask(&self) -> &Array2<bool> {
        &self.mask
    }

    #[must_use]
    pub fn nodata(&self) -> Option<f64> {
        self.nodata
    }

    #[must_use]
    pub fn bounds(&self) -> Option<[f64; 4]> {
        self.bounds
    }

    #[must_use]
    pub fn crs_epsg(&self) -> Option<u32> {
        self.crs_epsg
    }

    #[must_use]
    pub fn band_count(&self) -> usize {
        self.data.dim().0
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.data.dim().1
    }

    #[must_use]
    pub fn width(&self) -> usize {
        self.data.dim().2
    }

    /// Consume and return the underlying `(data, mask)` arrays.
    #[must_use]
    pub fn into_parts(self) -> (Array3<f32>, Array2<bool>) {
        (self.data, self.mask)
    }

    /// Borrow the data + mask planes simultaneously as mutable views.
    ///
    /// Enables mosaic methods to write into data and mask together
    /// without re-borrowing `self` (which would produce E0499).
    pub fn views_mut(
        &mut self,
    ) -> (
        ndarray::ArrayViewMut3<'_, f32>,
        ndarray::ArrayViewMut2<'_, bool>,
    ) {
        (self.data.view_mut(), self.mask.view_mut())
    }

    /// Returns true if every pixel is masked (fully transparent tile).
    /// Short-circuits on first unmasked pixel for O(1) early-out on
    /// typical inputs.
    #[must_use]
    pub fn is_fully_masked(&self) -> bool {
        self.mask.iter().all(|&m| m)
    }

    /// Returns true if no pixels are masked (fully opaque tile).
    /// Short-circuits on first masked pixel.
    #[must_use]
    pub fn is_fully_opaque(&self) -> bool {
        self.mask.iter().all(|&m| !m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn new_basic_shape() {
        let data = Array3::<f32>::zeros((3, 4, 5));
        let mask = Array2::from_elem((4, 5), false);
        let img = RasterImage::new(data, mask, None);
        assert_eq!(img.band_count(), 3);
        assert_eq!(img.height(), 4);
        assert_eq!(img.width(), 5);
        assert!(img.is_fully_opaque());
        assert!(!img.is_fully_masked());
    }

    #[test]
    fn from_opaque_sets_mask_false() {
        let data = array![[[1.0, 2.0], [3.0, 4.0]]];
        let img = RasterImage::from_opaque(data, Some(0.0));
        assert!(img.is_fully_opaque());
        assert_eq!(img.nodata(), Some(0.0));
    }

    #[test]
    fn is_fully_masked_detects_all_masked() {
        let data = Array3::<f32>::zeros((1, 2, 2));
        let mask = Array2::from_elem((2, 2), true);
        let img = RasterImage::new(data, mask, None);
        assert!(img.is_fully_masked());
        assert!(!img.is_fully_opaque());
    }

    #[test]
    fn metadata_round_trip() {
        let data = Array3::<f32>::zeros((2, 3, 4));
        let mask = Array2::from_elem((3, 4), false);
        let img = RasterImage::new(data, mask, Some(-9999.0))
            .with_bounds([-180.0, -90.0, 180.0, 90.0])
            .with_crs(4326);
        let meta = img.metadata();
        assert_eq!(meta.bands, 2);
        assert_eq!(meta.height, 3);
        assert_eq!(meta.width, 4);
        assert_eq!(meta.nodata, Some(-9999.0));
        assert_eq!(meta.bounds, Some([-180.0, -90.0, 180.0, 90.0]));
        assert_eq!(meta.crs_epsg, Some(4326));
    }

    #[test]
    #[should_panic(expected = "mask dimensions")]
    fn panic_on_mismatched_mask() {
        let data = Array3::<f32>::zeros((1, 4, 5));
        let mask = Array2::from_elem((3, 5), false);
        let _ = RasterImage::new(data, mask, None);
    }

    #[test]
    #[should_panic(expected = "band count must be >= 1")]
    fn panic_on_zero_bands() {
        let data = Array3::<f32>::zeros((0, 4, 5));
        let mask = Array2::from_elem((4, 5), false);
        let _ = RasterImage::new(data, mask, None);
    }
}
