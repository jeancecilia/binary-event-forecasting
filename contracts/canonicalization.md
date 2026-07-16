# Canonicalization

## Canonical JSON Serialization

All hashing uses canonical JSON serialization with the following rules:

1. **Sorted keys**: Object keys are sorted lexicographically by UTF-8 byte order.
2. **No whitespace**: No spaces, tabs, or newlines outside of string values.
3. **Minimal escaping**: Only characters that must be escaped per RFC 8259 are escaped.
4. **Consistent number formatting**: Numbers are serialized without unnecessary trailing zeros.
5. **UTC timestamps**: All timestamps are in ISO 8601 format with `Z` suffix.

## Hash Algorithm

`SHA-256` is used for all content hashing.

## Hash Computation Steps

1. Serialize the value to canonical JSON bytes.
2. Compute `SHA-256(canonical_json_bytes)`.
3. Encode as lowercase hexadecimal string.

## Cross-Language Hash Equivalence

Golden vectors in [`golden-vectors/`](golden-vectors/) must produce identical hashes in both Rust and Python.

```bash
just validate-contracts
```

## What Gets Hashed

- Forecast messages (before and after validation)
- Simulation intents (canonical inputs only, excluding runtime metadata)
- Journal records (hash chain: each record references previous record hash)
- Market snapshots
- Experiment manifests
- Configuration files
- Artifact store entries (content-addressed by SHA-256)

## What Is Excluded from Hashes

- Runtime telemetry timestamps
- Wall-clock measurements
- Random nonces (unless part of a deterministic derivation)
- Process/thread IDs
- Hostnames
- Filesystem metadata

## Unordered Collection Canonicalization

Collections (sets, maps) must be sorted before hashing:
- Sets: sort elements by their canonical JSON representation
- Maps: sort entries by key's canonical JSON representation

## Schema Version

Each schema version defines its own `ProbabilityScale` constant.
Schema version 1 uses `ProbabilityScale = 1_000_000`.
