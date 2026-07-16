//! Canonical JSON serialization and hashing.
//!
//! All hashing uses canonical JSON with sorted keys and no whitespace.
//! This ensures deterministic output across Rust and Python.

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// Compute the SHA-256 hash of a value using canonical JSON serialization.
pub fn canonical_hash<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let json = canonical_json(value)?;
    let hash = Sha256::digest(json.as_bytes());
    Ok(hex::encode(hash))
}

/// Serialize a value to canonical JSON.
///
/// Rules:
/// - Object keys are sorted lexicographically
/// - No whitespace outside string values
/// - Minimal Unicode escaping
/// - Consistent number formatting
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let json_value = serde_json::to_value(value)?;
    let canonical = sort_json_keys(&json_value);
    serde_json::to_string(&canonical)
}

/// Recursively sort all object keys in a JSON value.
fn sort_json_keys(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map
                .iter()
                .map(|(k, v)| (k.clone(), sort_json_keys(v)))
                .collect();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
            let mut sorted = Map::new();
            for (k, v) in entries {
                sorted.insert(k, v);
            }
            Value::Object(sorted)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(sort_json_keys).collect())
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestStruct {
        zebra: String,
        apple: i32,
        banana: bool,
    }

    #[test]
    fn test_keys_are_sorted() {
        let input = TestStruct {
            zebra: "z".to_string(),
            apple: 1,
            banana: true,
        };
        let json = canonical_json(&input).unwrap();
        // 'apple' must appear before 'banana' before 'zebra'
        let apple_pos = json.find("\"apple\"").unwrap();
        let banana_pos = json.find("\"banana\"").unwrap();
        let zebra_pos = json.find("\"zebra\"").unwrap();
        assert!(apple_pos < banana_pos);
        assert!(banana_pos < zebra_pos);
        // No whitespace
        assert!(!json.contains(" : "));
    }

    #[test]
    fn test_hash_is_deterministic() {
        let input = TestStruct {
            zebra: "z".to_string(),
            apple: 1,
            banana: true,
        };
        let hash1 = canonical_hash(&input).unwrap();
        let hash2 = canonical_hash(&input).unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_nested_objects_sorted() {
        let outer = serde_json::json!({
            "z": { "b": 2, "a": 1 },
            "a": 1,
        });
        let json = canonical_json(&outer).unwrap();
        assert!(json.find("\"a\"").unwrap() < json.find("\"z\"").unwrap());
        // Nested object keys also sorted
        let inner_start = json.find("\"z\"").unwrap();
        let inner = &json[inner_start..];
        assert!(inner.find("\"a\"").unwrap() < inner.find("\"b\"").unwrap());
    }
}
