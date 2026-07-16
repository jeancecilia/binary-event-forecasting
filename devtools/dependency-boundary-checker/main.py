import sys
import tomllib
from pathlib import Path

ALLOWED_INTERNAL_DEPS: dict[str, set[str]] = {
    "domain-types": set(),
    "protocol": {"domain-types"},
    "market-state": {"domain-types", "protocol"},
    "forecast-policy": {"domain-types", "protocol", "market-state"},
    "matching": {"domain-types", "protocol", "market-state"},
    "ledger": {"domain-types", "protocol", "matching"},
    "journal": {"domain-types", "protocol", "ledger"},
    "replay": {"domain-types", "protocol", "journal"},
    "experiment-control": {"protocol", "domain-types"},
    "telemetry": {"protocol", "domain-types"},
    "core-engine": {
        "domain-types", "protocol", "market-state", "forecast-policy", "matching",
        "ledger", "journal", "replay", "experiment-control", "telemetry"
    },
    "mock-gateway": {
        "domain-types", "protocol", "market-state", "forecast-policy", "matching",
        "ledger", "journal", "replay", "experiment-control", "telemetry"
    },
}


def check_dependencies():
    errors: list[str] = []
    workspace_root = Path(__file__).parent.parent.parent
    for toml_path in workspace_root.rglob("Cargo.toml"):
        if "target" in toml_path.parts:
            continue
        try:
            with open(toml_path, "rb") as f:
                data = tomllib.load(f)

            package_name = data.get("package", {}).get("name")
            if not package_name or package_name not in ALLOWED_INTERNAL_DEPS:
                continue

            allowed = ALLOWED_INTERNAL_DEPS[package_name]

            # Check standard dependencies
            deps = data.get("dependencies", {})
            for dep_name in deps:
                if dep_name in ALLOWED_INTERNAL_DEPS and dep_name not in allowed:
                    errors.append(
                        f"FAIL: {package_name} depends on {dep_name} (not allowed by architecture)"
                    )

            # Check dev-dependencies
            dev_deps = data.get("dev-dependencies", {})
            for dep_name in dev_deps:
                if dep_name in ALLOWED_INTERNAL_DEPS and dep_name not in allowed:
                    errors.append(
                        f"FAIL (dev): {package_name} depends on {dep_name} "
                        "(not allowed by architecture)"
                    )

        except Exception as e:
            errors.append(f"FAIL: parsing {toml_path} - {e}")

    if errors:
        for err in errors:
            print(err)
        sys.exit(1)

    print("Dependency boundary check passed.")
    sys.exit(0)


if __name__ == "__main__":
    check_dependencies()
