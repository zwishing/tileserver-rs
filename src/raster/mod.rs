//! Array-backed raster types for the STAC/COG/raster pipeline.
//!
//! The raster pipeline flows `ndarray::Array3<f32>` end-to-end so that
//! mosaic compositing, pixel selection, band math, and colormaps can all
//! operate on raw pixel values instead of PNG/JPEG bytes. PNG/JPEG only
//! appear at the HTTP response boundary.
//!
//! This is the Rust analogue of titiler's numpy-array pipeline: pixel
//! values flow as a dense array with an explicit validity mask, and each
//! transformation returns a new array rather than round-tripping through
//! a compressed codec.
//!
//! # Structure
//!
//! - [`RasterImage`]: dense `(bands, height, width)` float array + mask + metadata.
//! - [`encode`]: write `RasterImage` as PNG/JPEG/WebP bytes (HTTP boundary).
//! - [`decode`]: read PNG/JPEG bytes into a `RasterImage`.
//! - [`convert`]: lossless bridge between `RasterImage` and `image::RgbaImage`
//!   for interop with the legacy PNG-bytes pipeline during incremental migration.
//!
//! # Why not use `image::DynamicImage` end-to-end?
//!
//! `image::RgbaImage` is a `u8` packed byte array (not float), has no mask
//! channel, and cannot represent multi-band data beyond RGBA. Band math
//! on Sentinel-2 needs at minimum `f32` so precision is preserved across
//! `(B08 - B04) / (B08 + B04)`, which clips to zero in `u8` space.

pub mod convert;
pub mod decode;
pub mod encode;
pub mod expression;
pub mod image;
pub mod mosaic;

pub use expression::{ExpressionError, ParsedExpression};
pub use image::{RasterImage, RasterMetadata};
