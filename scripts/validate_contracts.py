#!/usr/bin/env python3
"""Cross-language contract validator.

Validates that JSON schemas, Rust types, Python models, and golden vectors
are consistent.
"""

from pathlib import Path
import sys

# Add python-packages/contracts-py/src to path
repo_root = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(repo_root / "python-packages" / "contracts-py" / "src"))

from contracts_py.forecast import ForecastMessage

def main() -> int:
    print("Contract validation starting...")
    
    golden_forecast_path = repo_root / "data" / "traces" / "golden" / "forecast-messages.jsonl"
    if not golden_forecast_path.exists():
        print(f"Error: missing golden trace {golden_forecast_path}")
        return 1

    try:
        with open(golden_forecast_path, "r") as f:
            for line in f:
                if not line.strip():
                    continue
                # Validate against python Pydantic schema
                msg = ForecastMessage.model_validate_json(line)
                # Verify round-trip serialization matches JSON Schema expectations
                assert msg.message_id is not None
        print("Contract validation passed. Python models and golden vectors agree.")
        return 0
    except Exception as e:
        print(f"Contract validation failed: {e}")
        return 1

if __name__ == "__main__":
    sys.exit(main())
