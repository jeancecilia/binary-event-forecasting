# Content-Addressed Artifact Storage

All models, calibration files, traces, manifests, and reports are stored by SHA-256.

## Layout

```
var/artifacts/sha256/
└── <first-2-char>/
    └── <next-2-char>/
        └── <full-64-char-hex-hash>
```

## Properties

- **Immutable**: Once written, never overwritten
- **Content-addressed**: Path is derived from content hash
- **Deduplicated**: Identical content stored once
- **Verifiable**: Read back and verify hash matches path

## Usage

1. Compute SHA-256 of the content bytes.
2. Write to `var/artifacts/sha256/<AB>/<CD>/<full-hash>`.
3. Store the hash in the database alongside metadata.

## Verification

```bash
sha256sum var/artifacts/sha256/ab/cd/abcdef...full-hash
```
