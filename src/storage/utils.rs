#![allow(dead_code)]
//! Shared SQLite parsing helpers for the storage layer.
//!
//! Centralizes lossless conversions commonly needed in rusqlite row mappers,
//! providing consistent error mapping and type safety.
//!
//! These helpers intentionally return `rusqlite::Error` so they can be used
//! inside `query_row/query_map` closures without additional error bridging.

use chrono::{DateTime, Utc};
use rusqlite::types::Type;
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// Parse a UUID from a string originating from SQLite, mapping errors
/// to rusqlite::Error so it composes naturally with row mappers.
///
/// - `s`: raw string value from SQLite (typically `row.get::<_, String>(idx)?`)
/// - `idx`: column index to report in error context
pub fn parse_uuid_sqlite(s: &str, idx: usize) -> Result<Uuid, rusqlite::Error> {
    Uuid::parse_str(s)
        .map_err(|_| rusqlite::Error::InvalidColumnType(idx, "uuid".to_string(), Type::Text))
}

/// Parse an optional UUID from an Option<String> originating from SQLite.
///
/// - `opt`: optional string value from SQLite (e.g., `row.get::<_, Option<String>>(idx)?`)
/// - `idx`: column index to report in error context
pub fn parse_optional_uuid_sqlite(
    opt: Option<String>,
    idx: usize,
) -> Result<Option<Uuid>, rusqlite::Error> {
    match opt {
        Some(s) => Uuid::parse_str(&s)
            .map(Some)
            .map_err(|_| rusqlite::Error::InvalidColumnType(idx, "uuid".to_string(), Type::Text)),
        None => Ok(None),
    }
}

/// Parse JSON from a string originating from SQLite into a strongly-typed value,
/// mapping parse failures into rusqlite::Error::InvalidColumnType for seamless use
/// within row mappers.
///
/// - `s`: JSON string from SQLite
/// - `idx`: column index to report in error context
pub fn parse_json_sqlite_at<T: DeserializeOwned>(
    s: &str,
    idx: usize,
) -> Result<T, rusqlite::Error> {
    serde_json::from_str(s)
        .map_err(|e| rusqlite::Error::InvalidColumnType(idx, format!("json: {e}"), Type::Text))
}

/// Convenience wrapper when the column index is unknown or irrelevant.
pub fn parse_json_sqlite<T: DeserializeOwned>(s: &str) -> Result<T, rusqlite::Error> {
    parse_json_sqlite_at(s, 0)
}

/// Parse RFC3339 datetime string (TEXT column) into `DateTime<Utc>`.
///
/// - `s`: RFC3339 timestamp string (e.g., "2024-01-02T03:04:05Z")
/// - `idx`: column index to report in error context
pub fn parse_datetime_rfc3339_sqlite(
    s: &str,
    idx: usize,
) -> Result<DateTime<Utc>, rusqlite::Error> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| {
            rusqlite::Error::InvalidColumnType(idx, "rfc3339-datetime".to_string(), Type::Text)
        })
}

/// Convert INTEGER epoch seconds (commonly stored in SQLite) to `DateTime<Utc>`.
///
/// - `ts`: unix epoch seconds
/// - `idx`: column index to report in error context
pub fn datetime_from_unix_seconds_at(
    ts: i64,
    idx: usize,
) -> Result<DateTime<Utc>, rusqlite::Error> {
    DateTime::from_timestamp(ts, 0).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(idx, "epoch-seconds".to_string(), Type::Integer)
    })
}

/// Convert an optional INTEGER epoch seconds into an Option<DateTime<Utc>>.
///
/// - `opt_ts`: optional unix epoch seconds
/// - `idx`: column index to report in error context
pub fn optional_datetime_from_unix_seconds_at(
    opt_ts: Option<i64>,
    idx: usize,
) -> Result<Option<DateTime<Utc>>, rusqlite::Error> {
    match opt_ts {
        Some(ts) => datetime_from_unix_seconds_at(ts, idx).map(Some),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_parse_uuid_ok() {
        let u = Uuid::new_v4();
        let parsed = parse_uuid_sqlite(&u.to_string(), 2).unwrap();
        assert_eq!(parsed, u);
    }

    #[test]
    fn test_parse_uuid_err() {
        let err = parse_uuid_sqlite("not-a-uuid", 5).err().unwrap();
        match err {
            rusqlite::Error::InvalidColumnType(idx, _, _) => assert_eq!(idx, 5),
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn test_parse_optional_uuid() {
        let u = Uuid::new_v4();
        assert_eq!(
            parse_optional_uuid_sqlite(Some(u.to_string()), 1).unwrap(),
            Some(u)
        );
        assert_eq!(parse_optional_uuid_sqlite(None, 1).unwrap(), None);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Foo {
        a: i32,
        b: String,
    }

    #[test]
    fn test_parse_json_ok() {
        let json = r#"{"a": 1, "b": "x"}"#;
        let val: Foo = parse_json_sqlite_at(json, 3).unwrap();
        assert_eq!(
            val,
            Foo {
                a: 1,
                b: "x".into()
            }
        );
    }

    #[test]
    fn test_parse_json_err() {
        let json = r#"{"a": "bad", "b": "x"}"#;
        let err = parse_json_sqlite_at::<Foo>(json, 7).err().unwrap();
        match err {
            rusqlite::Error::InvalidColumnType(idx, msg, _) => {
                assert_eq!(idx, 7);
                assert!(msg.starts_with("json:"));
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn test_parse_rfc3339_datetime_ok() {
        let dt = parse_datetime_rfc3339_sqlite("2024-01-02T03:04:05Z", 0).unwrap();
        assert_eq!(dt.timestamp(), 1704164645);
    }

    #[test]
    fn test_parse_rfc3339_datetime_err() {
        let err = parse_datetime_rfc3339_sqlite("02-01-2024 03:04:05", 4)
            .err()
            .unwrap();
        match err {
            rusqlite::Error::InvalidColumnType(idx, _, _) => assert_eq!(idx, 4),
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn test_datetime_from_unix_seconds() {
        let dt = datetime_from_unix_seconds_at(1_704_164_645, 2).unwrap();
        assert_eq!(dt.timestamp(), 1_704_164_645);
    }
}
