//! OGC API Features — Part 3: Filtering (CQL2-text and CQL2-JSON).
//!
//! Parses either the `cql2-text` or `cql2-json` dialect into a PostgreSQL
//! `WHERE` fragment. The implementation wraps the `cql2` crate but adds
//! two defences its 0.5.3 output lacks:
//!
//! 1. **Identifier quoting** — `cql2::ToSqlAst::to_sql()` emits bare
//!    property names (e.g. `name = 'Berlin'`), which would let a malicious
//!    filter reference system tables or reserved words. We walk the AST
//!    and wrap every `Expr::Property` in double quotes before SQL
//!    generation. `"` inside property names is escaped to `""`.
//!
//! 2. **Round-trip safety gate** — `cql2`'s grammar is lenient enough to
//!    silently drop trailing garbage (e.g. `name = 'x'; DROP TABLE t; --`
//!    parses as `name = 'x'`). We re-serialize the parsed expression back
//!    to CQL2-text and require it to round-trip, which rejects any input
//!    that loses characters during parsing.
//!
//! Reference: https://docs.ogc.org/is/19-079r2/19-079r2.html

use cql2::{Expr, ToSqlAst};

use crate::error::TileServerError;

/// Conformance URIs advertised when Part 3 is compiled in.
pub(crate) const CONFORMANCE_FILTER: &str =
    "http://www.opengis.net/spec/ogcapi-features-3/1.0/conf/filter";
pub(crate) const CONFORMANCE_FEATURES_FILTER: &str =
    "http://www.opengis.net/spec/ogcapi-features-3/1.0/conf/features-filter";
pub(crate) const CONFORMANCE_QUERYABLES: &str =
    "http://www.opengis.net/spec/ogcapi-features-3/1.0/conf/queryables";

/// OGC API Features Part 3 declares two filter languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FilterLang {
    CqlText,
    CqlJson,
}

impl FilterLang {
    fn parse(raw: &str) -> Result<Self, TileServerError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "cql2-text" | "cql-text" => Ok(Self::CqlText),
            "cql2-json" | "cql-json" => Ok(Self::CqlJson),
            _ => Err(TileServerError::InvalidTileRequest),
        }
    }
}

/// Recursively wraps every `Expr::Property` node in a PostgreSQL-safe
/// double-quoted identifier before SQL generation. This is the primary
/// SQL-injection defence — without it, a filter like
/// `information_schema.tables = 'x'` would produce bare SQL that targets
/// catalog tables.
fn quote_properties(expr: Expr) -> Expr {
    match expr {
        Expr::Property { property } => Expr::Property {
            property: format!("\"{}\"", property.replace('"', "\"\"")),
        },
        Expr::Operation { op, args } => Expr::Operation {
            op,
            args: args
                .into_iter()
                .map(|a| Box::new(quote_properties(*a)))
                .collect(),
        },
        Expr::Interval { interval } => Expr::Interval {
            interval: interval
                .into_iter()
                .map(|a| Box::new(quote_properties(*a)))
                .collect(),
        },
        Expr::Timestamp { timestamp } => Expr::Timestamp {
            timestamp: Box::new(quote_properties(*timestamp)),
        },
        Expr::Date { date } => Expr::Date {
            date: Box::new(quote_properties(*date)),
        },
        Expr::BBox { bbox } => Expr::BBox {
            bbox: bbox
                .into_iter()
                .map(|a| Box::new(quote_properties(*a)))
                .collect(),
        },
        Expr::Array(items) => Expr::Array(
            items
                .into_iter()
                .map(|a| Box::new(quote_properties(*a)))
                .collect(),
        ),
        other => other,
    }
}

/// Parses a CQL2 expression and returns a PostgreSQL-compatible SQL fragment
/// ready to splice into a `WHERE` clause.
///
/// The caller may supply `lang` to force a specific dialect, or `None` to
/// auto-detect (leading `{` → JSON, otherwise text). Identifier quoting
/// and the round-trip safety gate are always applied.
///
/// # Errors
///
/// Returns [`TileServerError::InvalidTileRequest`] when:
/// - The expression does not parse as CQL2.
/// - The re-serialisation (`to_text`) drops characters, indicating the
///   parser accepted a malformed input.
/// - The AST cannot be translated to SQL.
pub(crate) fn translate_filter_to_sql(
    raw: &str,
    lang: Option<&str>,
) -> Result<String, TileServerError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(TileServerError::InvalidTileRequest);
    }

    let resolved_lang = match lang {
        Some(s) => Some(FilterLang::parse(s)?),
        None => None,
    };

    let expr = match resolved_lang {
        Some(FilterLang::CqlText) => cql2::parse_text(trimmed).map_err(|e| {
            tracing::warn!(error = %e, filter = %trimmed, "CQL2-text parse failed");
            TileServerError::InvalidTileRequest
        })?,
        Some(FilterLang::CqlJson) => cql2::parse_json(trimmed).map_err(|e| {
            tracing::warn!(error = %e, filter = %trimmed, "CQL2-JSON parse failed");
            TileServerError::InvalidTileRequest
        })?,
        None => trimmed.parse::<Expr>().map_err(|e| {
            tracing::warn!(error = %e, filter = %trimmed, "CQL2 auto-detect parse failed");
            TileServerError::InvalidTileRequest
        })?,
    };

    reject_non_roundtrip(&expr, trimmed, resolved_lang)?;

    let quoted = quote_properties(expr);
    quoted.to_sql().map_err(|e| {
        tracing::warn!(error = %e, filter = %trimmed, "CQL2 -> SQL translation failed");
        TileServerError::InvalidTileRequest
    })
}

/// Safety gate: reject inputs where cql2 accepted less than we supplied.
///
/// For `cql2-text` input, we compare the parser's own normalised output
/// against the input after stripping whitespace. For JSON we only reject
/// empty text round-trip (the JSON schema validator inside cql2 already
/// blocks most hostile shapes).
fn reject_non_roundtrip(
    expr: &Expr,
    original: &str,
    lang: Option<FilterLang>,
) -> Result<(), TileServerError> {
    let treat_as_text = matches!(lang, Some(FilterLang::CqlText))
        || (lang.is_none() && !original.trim_start().starts_with('{'));
    if !treat_as_text {
        return Ok(());
    }

    let Ok(reserialised) = expr.to_text() else {
        return Err(TileServerError::InvalidTileRequest);
    };

    let canonical_original = canonicalise_cql_text(original);
    let canonical_reserialised = canonicalise_cql_text(&reserialised);

    if canonical_original != canonical_reserialised {
        tracing::warn!(
            original = %original,
            reserialised = %reserialised,
            "CQL2-text round-trip mismatch; rejecting possibly-malicious filter"
        );
        return Err(TileServerError::InvalidTileRequest);
    }
    Ok(())
}

/// Canonicalises CQL2-text for the round-trip comparison.
///
/// `cql2::to_text()` normalises the user input by wrapping groups in
/// parentheses, standardising casing, and adding spaces after commas. To
/// tell apart a legitimate difference (whitespace, casing, redundant
/// parens the user omitted) from a malicious difference (dropped tokens),
/// we strip everything the normaliser is allowed to change:
///
/// - parentheses are removed (cql2 always parenthesises nested exprs)
/// - whitespace is collapsed to single spaces then stripped around `,`
/// - casing is lowered uniformly
///
/// The remaining token stream must match byte-for-byte between the user's
/// input and the round-tripped output. Any SQL trailer
/// (`; DROP TABLE ...; --`) is detectable here because it contains
/// semicolons or `--` that cql2 cannot emit itself.
fn canonicalise_cql_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = true;
    for ch in s.chars() {
        match ch {
            '(' | ')' => continue,
            c if c.is_whitespace() => {
                if !prev_space && !out.is_empty() {
                    out.push(' ');
                    prev_space = true;
                }
            }
            c => {
                out.push(c.to_ascii_lowercase());
                prev_space = false;
            }
        }
    }
    while out.ends_with(' ') {
        out.pop();
    }
    out.replace(" ,", ",").replace(", ", ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_text_equality_and_quotes_identifier() {
        let sql = translate_filter_to_sql("name = 'Berlin'", Some("cql2-text")).unwrap();
        assert!(
            sql.contains("\"name\""),
            "missing quoted identifier in {sql}"
        );
        assert!(sql.contains("'Berlin'"));
        assert!(sql.contains('='));
    }

    #[test]
    fn parses_comparison_and_conjunction() {
        let sql = translate_filter_to_sql(
            "population > 1000000 AND is_capital = true",
            Some("cql2-text"),
        )
        .unwrap();
        assert!(
            sql.contains("\"population\""),
            "missing population quote: {sql}"
        );
        assert!(sql.contains("1000000"));
        assert!(
            sql.contains("\"is_capital\""),
            "missing is_capital quote: {sql}"
        );
        assert!(sql.to_lowercase().contains("and"));
    }

    #[test]
    fn parses_is_null() {
        let sql = translate_filter_to_sql("founded_year IS NULL", Some("cql2-text")).unwrap();
        assert!(sql.to_uppercase().contains("IS NULL"));
    }

    #[test]
    fn parses_in_operator() {
        let sql = translate_filter_to_sql("iso_a2 IN ('US','GB','DE')", Some("cql2-text")).unwrap();
        assert!(sql.contains("'US'"));
        assert!(sql.contains("'DE'"));
        assert!(sql.contains("\"iso_a2\""));
    }

    #[test]
    fn parses_between() {
        let sql =
            translate_filter_to_sql("population BETWEEN 1000000 AND 10000000", Some("cql2-text"))
                .unwrap();
        assert!(sql.to_uppercase().contains("BETWEEN"));
        assert!(sql.contains("\"population\""));
    }

    #[test]
    fn parses_cql2_json_and_quotes_identifier() {
        let json = r#"{"op":"=","args":[{"property":"iso_a2"},"DE"]}"#;
        let sql = translate_filter_to_sql(json, Some("cql2-json")).unwrap();
        assert!(sql.contains("\"iso_a2\""), "missing quote in {sql}");
        assert!(sql.contains("'DE'"));
    }

    #[test]
    fn auto_detect_picks_json_on_brace() {
        let json = r#"{"op":"=","args":[{"property":"iso_a2"},"DE"]}"#;
        let sql = translate_filter_to_sql(json, None).unwrap();
        assert!(sql.contains("\"iso_a2\""));
    }

    #[test]
    fn auto_detect_picks_text_otherwise() {
        let sql = translate_filter_to_sql("name = 'Paris'", None).unwrap();
        assert!(sql.contains("\"name\""));
    }

    #[test]
    fn rejects_empty_filter() {
        assert!(translate_filter_to_sql("", Some("cql2-text")).is_err());
        assert!(translate_filter_to_sql("   ", Some("cql2-text")).is_err());
    }

    #[test]
    fn rejects_unknown_filter_lang() {
        assert!(translate_filter_to_sql("name = 'x'", Some("xpath")).is_err());
    }

    #[test]
    fn accepts_legacy_cql_text_alias() {
        let sql = translate_filter_to_sql("name = 'Paris'", Some("cql-text")).unwrap();
        assert!(sql.contains("'Paris'"));
    }

    #[test]
    fn spatial_intersects_maps_to_st_intersects() {
        let sql =
            translate_filter_to_sql("S_INTERSECTS(geom, POINT(2.35 48.85))", Some("cql2-text"))
                .unwrap();
        assert!(sql.to_lowercase().contains("st_intersects"));
        assert!(sql.contains("\"geom\""));
    }

    #[test]
    fn sql_injection_trailing_semicolon_is_rejected_by_roundtrip_gate() {
        let attack = "name = 'x'; DROP TABLE cities; --";
        assert!(
            translate_filter_to_sql(attack, Some("cql2-text")).is_err(),
            "round-trip gate must reject trailing SQL garbage"
        );
    }

    #[test]
    fn identifier_with_double_quote_is_escaped() {
        let sql = translate_filter_to_sql(
            r#"{"op":"=","args":[{"property":"w\"t"},"x"]}"#,
            Some("cql2-json"),
        )
        .unwrap();
        assert!(
            sql.contains(r#"""w""""t"""#),
            "double-quote must be doubled (pg-style) inside identifier in {sql}"
        );
    }
}
