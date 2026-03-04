//! MLT (MapLibre Tiles) transcoding module.
//!
//! Provides conversion between MVT (Mapbox Vector Tiles, protobuf)
//! and MLT (MapLibre Tiles) formats. This enables:
//!
//! - **Phase 2**: Serve existing MVT/PBF sources as MLT tiles (MVT→MLT encoding)
//! - **Phase 3**: Serve MLT sources as MVT/PBF for backward compatibility with legacy clients
//!
//! Gated behind the `mlt` cargo feature.

use bytes::Bytes;
use flate2::read::GzDecoder;
use std::io::Read;

use crate::error::{Result, TileServerError};
use crate::sources::{TileCompression, TileData, TileFormat};

// ---------------------------------------------------------------------------
// MVT Protobuf types (minimal prost-generated structs for encoding MVT tiles)
// ---------------------------------------------------------------------------

/// Minimal MVT protobuf types for encoding vector tiles.
///
/// These mirror the Mapbox Vector Tile specification v2.1 protobuf schema.
/// Used only for MLT→MVT reverse transcoding (Phase 3).
#[allow(non_snake_case)]
pub mod MvtProto {
    /// MVT geometry type.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, prost::Enumeration)]
    #[repr(i32)]
    pub enum GeomType {
        Unknown = 0,
        Point = 1,
        Linestring = 2,
        Polygon = 3,
    }

    /// MVT property value.
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Value {
        #[prost(string, optional, tag = "1")]
        pub string_value: Option<String>,
        #[prost(float, optional, tag = "2")]
        pub float_value: Option<f32>,
        #[prost(double, optional, tag = "3")]
        pub double_value: Option<f64>,
        #[prost(int64, optional, tag = "4")]
        pub int_value: Option<i64>,
        #[prost(uint64, optional, tag = "5")]
        pub uint_value: Option<u64>,
        #[prost(sint64, optional, tag = "6")]
        pub sint_value: Option<i64>,
        #[prost(bool, optional, tag = "7")]
        pub bool_value: Option<bool>,
    }

    /// MVT feature within a layer.
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Feature {
        #[prost(uint64, optional, tag = "1")]
        pub id: Option<u64>,
        #[prost(uint32, repeated, packed = "true", tag = "2")]
        pub tags: Vec<u32>,
        #[prost(enumeration = "GeomType", optional, tag = "3")]
        pub r#type: Option<i32>,
        #[prost(uint32, repeated, packed = "true", tag = "4")]
        pub geometry: Vec<u32>,
    }

    /// MVT layer within a tile.
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Layer {
        #[prost(uint32, required, tag = "15")]
        pub version: u32,
        #[prost(string, required, tag = "1")]
        pub name: String,
        #[prost(message, repeated, tag = "2")]
        pub features: Vec<Feature>,
        #[prost(string, repeated, tag = "3")]
        pub keys: Vec<String>,
        #[prost(message, repeated, tag = "4")]
        pub values: Vec<Value>,
        #[prost(uint32, optional, tag = "5")]
        pub extent: Option<u32>,
    }

    /// MVT tile (top-level message).
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Tile {
        #[prost(message, repeated, tag = "3")]
        pub layers: Vec<Layer>,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Transcode a tile from its current format to a target format.
///
/// Handles decompression of source data if needed (gzip).
/// Returns a new `TileData` with the transcoded bytes and appropriate format.
///
/// # Supported conversions
///
/// | From | To  | Description |
/// |------|-----|-------------|
/// | PBF  | MLT | MVT→MLT encoding via mlt-core (Phase 2) |
/// | MLT  | PBF | MLT→MVT decoding (Phase 3) |
///
/// # Errors
///
/// Returns `TileServerError::TranscodeUnsupported` for unsupported format pairs.
/// Returns `TileServerError::MltEncodeError` or `MltDecodeError` on conversion failure.
pub fn transcode_tile(tile: &TileData, target_format: TileFormat) -> Result<TileData> {
    // No-op if formats already match
    if tile.format == target_format {
        return Ok(tile.clone());
    }

    match (tile.format, target_format) {
        (TileFormat::Pbf, TileFormat::Mlt) => {
            // Phase 2: MVT→MLT encoding using mlt-core's encoding API.
            let raw = decompress_tile_data(tile)?;
            let mlt_bytes = mvt_to_mlt(&raw)?;
            Ok(TileData {
                data: mlt_bytes,
                format: TileFormat::Mlt,
                compression: TileCompression::None,
            })
        }
        (TileFormat::Mlt, TileFormat::Pbf) => {
            let raw = decompress_tile_data(tile)?;
            let mvt_bytes = mlt_to_mvt(&raw)?;
            Ok(TileData {
                data: mvt_bytes,
                format: TileFormat::Pbf,
                compression: TileCompression::None,
            })
        }
        (from, to) => Err(TileServerError::TranscodeUnsupported {
            from: format!("{from:?}"),
            to: format!("{to:?}"),
        }),
    }
}

// ---------------------------------------------------------------------------
// Internal: Decompression
// ---------------------------------------------------------------------------

/// Decompress tile data if compressed, returning raw bytes.
fn decompress_tile_data(tile: &TileData) -> Result<Vec<u8>> {
    match tile.compression {
        TileCompression::None => Ok(tile.data.to_vec()),
        TileCompression::Gzip => {
            let mut decoder = GzDecoder::new(tile.data.as_ref());
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| {
                TileServerError::MltDecodeError(format!("gzip decompression failed: {e}"))
            })?;
            Ok(decompressed)
        }
        _ => Err(TileServerError::MltDecodeError(format!(
            "{:?} decompression not supported for transcoding",
            tile.compression
        ))),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: MVT → MLT encoding
// ---------------------------------------------------------------------------

/// Convert MVT (protobuf) bytes to MLT format.
///
/// Uses `mlt-core` to:
/// 1. Parse MVT binary into a `FeatureCollection`
/// 2. Group features by layer (via `_layer` property)
/// 3. Build decoded geometry, IDs, and column-oriented properties per layer
/// 4. Encode each column using mlt-core's encoding API
/// 5. Serialize encoded layers to MLT wire format
fn mvt_to_mlt(mvt_bytes: &[u8]) -> Result<Bytes> {
    use std::collections::BTreeMap;

    // Step 1: Parse MVT protobuf into FeatureCollection
    let fc = mlt_core::mvt::mvt_to_feature_collection(mvt_bytes.to_vec())
        .map_err(|e| TileServerError::MltEncodeError(format!("failed to parse MVT tile: {e}")))?;

    // Step 2: Group features by layer name
    let mut layer_map: BTreeMap<String, Vec<&mlt_core::geojson::Feature>> = BTreeMap::new();
    for feature in &fc.features {
        let layer_name = feature
            .properties
            .get("_layer")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        layer_map.entry(layer_name).or_default().push(feature);
    }

    // Step 3: Build and encode each layer, write to output buffer
    let mut output = Vec::new();
    for (layer_name, features) in &layer_map {
        let layer = build_mlt_layer(layer_name, features)?;
        layer.write_to(&mut output).map_err(|e| {
            TileServerError::MltEncodeError(format!("failed to write MLT layer: {e}"))
        })?;
    }

    Ok(Bytes::from(output))
}

/// Build an encoded `OwnedLayer` from a set of features belonging to one MVT layer.
fn build_mlt_layer(
    layer_name: &str,
    features: &[&mlt_core::geojson::Feature],
) -> Result<mlt_core::OwnedLayer> {
    use mlt_core::v01::{
        DecodedGeometry, DecodedId, Encoder, GeometryEncoder, IdEncoder, IdWidth, LogicalEncoder,
        OwnedGeometry, OwnedId, OwnedLayer01, OwnedProperty, PhysicalEncoder, PresenceStream,
        PropertyEncoder,
    };
    use mlt_core::Encodable as _;

    // Extract extent from first feature (injected by mvt_to_feature_collection as _extent)
    let extent = features
        .first()
        .and_then(|f| f.properties.get("_extent"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(4096);

    // --- Geometry ---
    let mut decoded_geom = DecodedGeometry::default();
    for feature in features {
        decoded_geom.push_geom(&feature.geometry);
    }
    let mut geometry = OwnedGeometry::Decoded(decoded_geom);
    let geom_encoder = GeometryEncoder::all(Encoder::varint());
    geometry.encode_with(geom_encoder).map_err(|e| {
        TileServerError::MltEncodeError(format!("failed to encode MLT geometry: {e}"))
    })?;

    // --- IDs ---
    let ids: Vec<Option<u64>> = features.iter().map(|f| f.id).collect();
    let has_ids = ids.iter().any(|id| id.is_some());
    let mut id = if has_ids {
        OwnedId::Decoded(DecodedId(Some(ids)))
    } else {
        OwnedId::None
    };
    if has_ids {
        let id_encoder = IdEncoder::new(LogicalEncoder::None, IdWidth::Id64);
        id.encode_with(id_encoder).map_err(|e| {
            TileServerError::MltEncodeError(format!("failed to encode MLT IDs: {e}"))
        })?;
    }

    // --- Properties (row-oriented → column-oriented) ---
    let properties = build_column_properties(features)?;
    let prop_encoder = PropertyEncoder::new(
        PresenceStream::Present,
        LogicalEncoder::None,
        PhysicalEncoder::VarInt,
    );
    let mut encoded_properties: Vec<OwnedProperty> = Vec::with_capacity(properties.len());
    for decoded_prop in properties {
        let mut prop = OwnedProperty::Decoded(decoded_prop);
        prop.encode_with(prop_encoder).map_err(|e| {
            TileServerError::MltEncodeError(format!("failed to encode MLT property: {e}"))
        })?;
        encoded_properties.push(prop);
    }

    // --- Assemble Layer ---
    let layer01 = OwnedLayer01 {
        name: layer_name.to_string(),
        extent,
        id,
        geometry,
        properties: encoded_properties,
    };

    Ok(mlt_core::OwnedLayer::Tag01(layer01))
}

/// Convert row-oriented feature properties to column-oriented `DecodedProperty` vectors.
///
/// MVT stores properties per-feature (row-oriented), but MLT stores them per-column.
/// This function collects all unique property keys, infers their types, and builds
/// a `PropValue` vector for each key across all features.
fn build_column_properties(
    features: &[&mlt_core::geojson::Feature],
) -> Result<Vec<mlt_core::v01::DecodedProperty>> {
    use mlt_core::v01::DecodedProperty;
    use std::collections::BTreeSet;

    // Collect all unique property keys (excluding internal _layer, _extent)
    let mut all_keys = BTreeSet::new();
    for feature in features {
        for key in feature.properties.keys() {
            if !key.starts_with('_') {
                all_keys.insert(key.clone());
            }
        }
    }

    let num_features = features.len();
    let mut result = Vec::with_capacity(all_keys.len());

    for key in &all_keys {
        // Determine the dominant type for this key by scanning feature values
        let prop_value = infer_column_values(features, key, num_features);
        result.push(DecodedProperty {
            name: key.clone(),
            values: prop_value,
        });
    }

    Ok(result)
}

/// Infer the column type and build a `PropValue` vector for a single property key.
///
/// Scans all features to determine the best type (String, i64, f64, bool),
/// inserting `None` for features where the property is absent.
fn infer_column_values(
    features: &[&mlt_core::geojson::Feature],
    key: &str,
    _num_features: usize,
) -> mlt_core::v01::PropValue {
    use mlt_core::v01::PropValue;

    // First pass: determine dominant type
    let mut has_string = false;
    let mut has_float = false;
    let mut has_int = false;
    let mut has_bool = false;

    for feature in features {
        if let Some(val) = feature.properties.get(key) {
            match val {
                serde_json::Value::String(_) => has_string = true,
                serde_json::Value::Bool(_) => has_bool = true,
                serde_json::Value::Number(n) => {
                    if n.is_f64() && !n.is_i64() && !n.is_u64() {
                        has_float = true;
                    } else {
                        has_int = true;
                    }
                }
                _ => {}
            }
        }
    }

    // If mixed types exist, prefer string (can represent anything)
    // Otherwise use the most specific type
    if has_string || (!has_int && !has_float && !has_bool) {
        let vals: Vec<Option<String>> = features
            .iter()
            .map(|f| {
                f.properties.get(key).and_then(|v| match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Null => None,
                    other => Some(other.to_string()),
                })
            })
            .collect();
        PropValue::Str(vals)
    } else if has_bool && !has_int && !has_float {
        let vals: Vec<Option<bool>> = features
            .iter()
            .map(|f| f.properties.get(key).and_then(|v| v.as_bool()))
            .collect();
        PropValue::Bool(vals)
    } else if has_float {
        let vals: Vec<Option<f64>> = features
            .iter()
            .map(|f| f.properties.get(key).and_then(|v| v.as_f64()))
            .collect();
        PropValue::F64(vals)
    } else {
        // Integer — use i64 since MVT properties can be signed
        let vals: Vec<Option<i64>> = features
            .iter()
            .map(|f| f.properties.get(key).and_then(|v| v.as_i64()))
            .collect();
        PropValue::I64(vals)
    }
}

// ---------------------------------------------------------------------------
// Phase 3: MLT → MVT decoding
// ---------------------------------------------------------------------------

/// Convert MLT bytes to MVT (protobuf) format.
///
/// Uses `mlt-core` to:
/// 1. Parse MLT binary into layers
/// 2. Decode all columns (geometry, properties, IDs)
/// 3. Convert decoded features to intermediate `FeatureCollection`
/// 4. Build MVT protobuf from features using prost
fn mlt_to_mvt(mlt_bytes: &[u8]) -> Result<Bytes> {
    use prost::Message;

    // Step 1: Parse MLT layers (lazy — column data not yet decoded)
    let mut layers = mlt_core::parse_layers(mlt_bytes)
        .map_err(|e| TileServerError::MltDecodeError(format!("failed to parse MLT tile: {e}")))?;

    // Step 2: Decode all columns in each layer
    for layer in &mut layers {
        layer.decode_all().map_err(|e| {
            TileServerError::MltDecodeError(format!("failed to decode MLT layer: {e}"))
        })?;
    }

    // Step 3: Convert decoded MLT layers to FeatureCollection
    let fc = mlt_core::geojson::FeatureCollection::from_layers(&layers).map_err(|e| {
        TileServerError::MltDecodeError(format!("failed to convert MLT layers to features: {e}"))
    })?;

    // Step 4: Build MVT protobuf from FeatureCollection
    let mvt_tile = feature_collection_to_mvt(&fc)?;

    // Step 5: Encode MVT protobuf to bytes
    let encoded = mvt_tile.encode_to_vec();
    Ok(Bytes::from(encoded))
}

/// Convert an MLT `FeatureCollection` to an MVT protobuf `Tile`.
///
/// Maps the GeoJSON-like features (with i32 tile coordinates) back into
/// the MVT protobuf wire format with keys/values interning.
///
/// Features are grouped by the `_layer` property injected by `from_layers`.
fn feature_collection_to_mvt(fc: &mlt_core::geojson::FeatureCollection) -> Result<MvtProto::Tile> {
    use std::collections::HashMap;

    let mut mvt_layers: Vec<MvtProto::Layer> = Vec::new();

    // Group features by layer name (stored in _layer property by from_layers)
    let mut layer_map: HashMap<&str, Vec<&mlt_core::geojson::Feature>> = HashMap::new();
    for feature in &fc.features {
        let layer_name = feature
            .properties
            .get("_layer")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        layer_map.entry(layer_name).or_default().push(feature);
    }

    for (layer_name, features) in &layer_map {
        // Get extent from first feature (injected by from_layers as _extent)
        let extent = features
            .first()
            .and_then(|f| f.properties.get("_extent"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(4096);

        let mut keys: Vec<String> = Vec::new();
        let mut values: Vec<MvtProto::Value> = Vec::new();
        let mut key_index: HashMap<String, u32> = HashMap::new();
        let mut value_index: HashMap<String, u32> = HashMap::new();
        let mut mvt_features: Vec<MvtProto::Feature> = Vec::new();

        for feature in features {
            // Encode geometry to MVT command sequence
            let (geom_type, geometry) = encode_geometry_to_mvt(&feature.geometry);

            // Encode properties as interned key/value tags
            // Skip internal properties (_layer, _extent)
            let mut tags = Vec::new();
            for (key, val) in &feature.properties {
                if key.starts_with('_') {
                    continue; // Skip internal properties
                }

                let key_idx = *key_index.entry(key.clone()).or_insert_with(|| {
                    let idx = keys.len() as u32;
                    keys.push(key.clone());
                    idx
                });

                let val_str = format!("{val}");
                let val_idx = *value_index.entry(val_str.clone()).or_insert_with(|| {
                    let idx = values.len() as u32;
                    values.push(json_value_to_mvt(val));
                    idx
                });

                tags.push(key_idx);
                tags.push(val_idx);
            }

            mvt_features.push(MvtProto::Feature {
                id: feature.id,
                tags,
                r#type: Some(geom_type as i32),
                geometry,
            });
        }

        mvt_layers.push(MvtProto::Layer {
            version: 2,
            name: layer_name.to_string(),
            features: mvt_features,
            keys,
            values,
            extent: Some(extent),
        });
    }

    Ok(MvtProto::Tile { layers: mvt_layers })
}

// ---------------------------------------------------------------------------
// MVT Geometry encoding helpers
// ---------------------------------------------------------------------------

/// Encode geo_types Geometry<i32> to MVT command-encoded geometry.
///
/// Returns `(GeomType, Vec<u32>)` with the MVT command sequence.
fn encode_geometry_to_mvt(geometry: &mlt_core::geojson::Geom32) -> (MvtProto::GeomType, Vec<u32>) {
    use geo_types::Geometry;

    match geometry {
        Geometry::Point(point) => {
            let commands = encode_point(point.x(), point.y());
            (MvtProto::GeomType::Point, commands)
        }
        Geometry::MultiPoint(mp) => {
            let points = mp.0.as_slice();
            let mut commands = Vec::with_capacity(points.len() * 3);
            // MoveTo(count)
            commands.push(command_integer(1, points.len() as u32));
            let mut cx = 0i32;
            let mut cy = 0i32;
            for point in points {
                let dx = point.x() - cx;
                let dy = point.y() - cy;
                commands.push(zigzag_encode(dx));
                commands.push(zigzag_encode(dy));
                cx = point.x();
                cy = point.y();
            }
            (MvtProto::GeomType::Point, commands)
        }
        Geometry::LineString(ls) => {
            let commands = encode_linestring(&ls.0, false);
            (MvtProto::GeomType::Linestring, commands)
        }
        Geometry::MultiLineString(mls) => {
            let mut commands = Vec::new();
            for line in &mls.0 {
                commands.extend(encode_linestring(&line.0, false));
            }
            (MvtProto::GeomType::Linestring, commands)
        }
        Geometry::Polygon(poly) => {
            let mut commands = Vec::new();
            // Exterior ring
            commands.extend(encode_linestring(&poly.exterior().0, true));
            // Interior rings (holes)
            for ring in poly.interiors() {
                commands.extend(encode_linestring(&ring.0, true));
            }
            (MvtProto::GeomType::Polygon, commands)
        }
        Geometry::MultiPolygon(mp) => {
            let mut commands = Vec::new();
            for polygon in &mp.0 {
                commands.extend(encode_linestring(&polygon.exterior().0, true));
                for ring in polygon.interiors() {
                    commands.extend(encode_linestring(&ring.0, true));
                }
            }
            (MvtProto::GeomType::Polygon, commands)
        }
        // Unsupported geometry types produce empty geometry
        _ => (MvtProto::GeomType::Unknown, Vec::new()),
    }
}

/// Encode a single point as MVT commands: MoveTo(1) + dx, dy.
fn encode_point(x: i32, y: i32) -> Vec<u32> {
    vec![
        command_integer(1, 1), // MoveTo, count=1
        zigzag_encode(x),
        zigzag_encode(y),
    ]
}

/// Encode a linestring/ring as MVT commands.
///
/// If `close_path` is true, appends ClosePath command (for polygon rings).
fn encode_linestring(coords: &[geo_types::Coord<i32>], close_path: bool) -> Vec<u32> {
    if coords.is_empty() {
        return Vec::new();
    }

    let mut commands = Vec::with_capacity(coords.len() * 2 + 4);
    let mut cx = 0i32;
    let mut cy = 0i32;

    // MoveTo first point
    commands.push(command_integer(1, 1));
    let dx = coords[0].x - cx;
    let dy = coords[0].y - cy;
    commands.push(zigzag_encode(dx));
    commands.push(zigzag_encode(dy));
    cx = coords[0].x;
    cy = coords[0].y;

    // LineTo remaining points
    let line_count = if close_path {
        // For closed rings, skip the last point (ClosePath handles it)
        coords.len().saturating_sub(2)
    } else {
        coords.len() - 1
    };

    if line_count > 0 {
        commands.push(command_integer(2, line_count as u32));
        let end = if close_path {
            coords.len() - 1
        } else {
            coords.len()
        };
        for coord in &coords[1..end] {
            let dx = coord.x - cx;
            let dy = coord.y - cy;
            commands.push(zigzag_encode(dx));
            commands.push(zigzag_encode(dy));
            cx = coord.x;
            cy = coord.y;
        }
    }

    // ClosePath for polygon rings
    if close_path {
        commands.push(command_integer(7, 1));
    }

    commands
}

/// Encode an MVT command integer: `(id & 0x7) | (count << 3)`.
#[inline]
fn command_integer(id: u32, count: u32) -> u32 {
    (id & 0x7) | (count << 3)
}

/// Zigzag-encode a signed integer for protobuf.
#[inline]
fn zigzag_encode(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

// ---------------------------------------------------------------------------
// MVT Value encoding helpers
// ---------------------------------------------------------------------------

/// Convert a serde_json::Value to an MVT protobuf Value.
fn json_value_to_mvt(val: &serde_json::Value) -> MvtProto::Value {
    match val {
        serde_json::Value::String(s) => MvtProto::Value {
            string_value: Some(s.clone()),
            ..Default::default()
        },
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                MvtProto::Value {
                    int_value: Some(i),
                    ..Default::default()
                }
            } else if let Some(u) = n.as_u64() {
                MvtProto::Value {
                    uint_value: Some(u),
                    ..Default::default()
                }
            } else if let Some(f) = n.as_f64() {
                MvtProto::Value {
                    double_value: Some(f),
                    ..Default::default()
                }
            } else {
                MvtProto::Value::default()
            }
        }
        serde_json::Value::Bool(b) => MvtProto::Value {
            bool_value: Some(*b),
            ..Default::default()
        },
        _ => MvtProto::Value::default(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_encode() {
        assert_eq!(zigzag_encode(0), 0);
        assert_eq!(zigzag_encode(-1), 1);
        assert_eq!(zigzag_encode(1), 2);
        assert_eq!(zigzag_encode(-2), 3);
        assert_eq!(zigzag_encode(2), 4);
    }

    #[test]
    fn test_command_integer() {
        // MoveTo, count=1
        assert_eq!(command_integer(1, 1), 9);
        // LineTo, count=3
        assert_eq!(command_integer(2, 3), 26);
        // ClosePath, count=1
        assert_eq!(command_integer(7, 1), 15);
    }

    #[test]
    fn test_encode_point() {
        let commands = encode_point(25, 17);
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], command_integer(1, 1)); // MoveTo(1)
        assert_eq!(commands[1], zigzag_encode(25)); // dx=25
        assert_eq!(commands[2], zigzag_encode(17)); // dy=17
    }

    #[test]
    fn test_transcode_same_format_is_noop() {
        let tile = TileData {
            data: Bytes::from_static(b"test"),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Pbf).unwrap();
        assert_eq!(result.data, tile.data);
        assert_eq!(result.format, TileFormat::Pbf);
    }

    #[test]
    fn test_transcode_unsupported_pair() {
        let tile = TileData {
            data: Bytes::from_static(b"test"),
            format: TileFormat::Png,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt);
        assert!(result.is_err());
        match result.unwrap_err() {
            TileServerError::TranscodeUnsupported { from, to } => {
                assert_eq!(from, "Png");
                assert_eq!(to, "Mlt");
            }
            e => panic!("expected TranscodeUnsupported, got: {e:?}"),
        }
    }

    #[test]
    fn test_mvt_to_mlt_invalid_input_returns_error() {
        // Invalid protobuf bytes should return an encode error, not panic
        let tile = TileData {
            data: Bytes::from_static(b"not valid protobuf"),
            format: TileFormat::Pbf,
            compression: TileCompression::None,
        };
        let result = transcode_tile(&tile, TileFormat::Mlt);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TileServerError::MltEncodeError(_)
        ));
    }

    #[test]
    fn test_json_value_to_mvt_string() {
        let val = serde_json::json!("hello");
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.string_value, Some("hello".to_string()));
    }

    #[test]
    fn test_json_value_to_mvt_number() {
        let val = serde_json::json!(42);
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.int_value, Some(42));
    }

    #[test]
    fn test_json_value_to_mvt_bool() {
        let val = serde_json::json!(true);
        let mvt = json_value_to_mvt(&val);
        assert_eq!(mvt.bool_value, Some(true));
    }
}
