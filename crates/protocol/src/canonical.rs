//! Canonical JSON serialization and hashing.
//!
//! All hashing uses canonical JSON with sorted keys and no whitespace.
//! This ensures deterministic output across Rust and Python.

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// Compute the SHA-256 hash of a value using canonical JSON serialization.
#[allow(dead_code, unreachable_pub)]
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
#[allow(dead_code, unreachable_pub)]
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
    fn test_keys_are_sorted() -> Result<(), Box<dyn std::error::Error>> {
        let input = TestStruct {
            zebra: "z".to_string(),
            apple: 1,
            banana: true,
        };
        let json = canonical_json(&input)?;
        let apple_pos = json.find("\"apple\"").ok_or("apple not found")?;
        let banana_pos = json.find("\"banana\"").ok_or("banana not found")?;
        let zebra_pos = json.find("\"zebra\"").ok_or("zebra not found")?;
        assert!(apple_pos < banana_pos);
        assert!(banana_pos < zebra_pos);
        assert!(!json.contains(" : "));
        Ok(())
    }

    #[test]
    fn test_hash_is_deterministic() -> Result<(), Box<dyn std::error::Error>> {
        let input = TestStruct {
            zebra: "z".to_string(),
            apple: 1,
            banana: true,
        };
        let hash1 = canonical_hash(&input)?;
        let hash2 = canonical_hash(&input)?;
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
        Ok(())
    }

    #[test]
    fn test_nested_objects_sorted() -> Result<(), Box<dyn std::error::Error>> {
        let outer = serde_json::json!({
            "z": { "b": 2, "a": 1 },
            "a": 1,
        });
        let json = canonical_json(&outer)?;
        let a_pos = json.find("\"a\"").ok_or("a not found")?;
        let z_pos = json.find("\"z\"").ok_or("z not found")?;
        assert!(a_pos < z_pos);
        let inner_start = z_pos;
        let inner = &json[inner_start..];
        let inner_a = inner.find("\"a\"").ok_or("inner a not found")?;
        let inner_b = inner.find("\"b\"").ok_or("inner b not found")?;
        assert!(inner_a < inner_b);
        Ok(())
    }
}
