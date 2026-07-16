# Trace Inspector

Tool for inspecting and validating replay traces.

## Usage

```bash
uv run python devtools/trace-inspector/main.py --trace data/traces/golden --validate
```

## Features

- Validate Parquet schema
- Check bitemporal consistency
- Verify event ordering
- Compute trace hash
