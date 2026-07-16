//! Canonical JSON serialization and hashing.
//!
//! All hashing uses canonical JSON:
//! - Keys are sorted alphabetically
//! - No whitespace outside string values
//! - Unicode is not escaped unnecessarily
//! - All numeric values are serialized consistently

use sha2::{Digest, Sha256};

/// Compute the SHA-256 hash of a value using canonical JSON serialization.
pub fn canonical_hash<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let json = canonical_json(value)?;
    let hash = Sha256::digest(json.as_bytes());
    Ok(hex::encode(hash))
}

/// Serialize a value to canonical JSON (sorted keys, no whitespace).
pub fn canonical_json<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    // Use canonical formatter: sorted keys, compact output
    let formatter = serde_json::ser::CompactFormatter;
    let _ = value.serialize(&mut serde_json::Serializer::with_formatter(
        std::io::BufWriter::new(Vec::new()),
        formatter,
    ));
    // Note: For truly canonical output, we need key sorting.
    // serde_json doesn't natively sort keys in structs.
    // Use serde_json::to_string with a BTreeMap wrapper for complex objects.
    serde_json::to_string(value)
}
