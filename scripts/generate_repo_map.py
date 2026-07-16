#!/usr/bin/env python3
"""Generate repository map.

Creates docs/generated/repository-map.md with module, symbol,
requirement, and test information.

Usage:
    python scripts/generate_repo_map.py
"""

import sys
from pathlib import Path


def main() -> int:
    repo_root = Path('.')
    output_path = repo_root / 'docs' / 'generated' / 'repository-map.md'
    output_path.parent.mkdir(parents=True, exist_ok=True)

    lines = [
        '# Repository Map',
        '',
        'Auto-generated module index.',
        '',
        '| Module | Responsibility | Public Symbols | Dependencies | REQ IDs | Tests |',
        '|--------|---------------|----------------|--------------|---------|-------|',
    ]

    # Scan crates
    crates_dir = repo_root / 'crates'
    if crates_dir.exists():
        for crate in sorted(crates_dir.iterdir()):
            if crate.is_dir():
                lines.append(
                    f'| `crates/{crate.name}/` | (stub) | — | — | — | — |'
                )

    # Scan services
    services_dir = repo_root / 'services'
    if services_dir.exists():
        for service in sorted(services_dir.iterdir()):
            if service.is_dir():
                lines.append(
                    f'| `services/{service.name}/` | (stub) | — | — | — | — |'
                )

    # Scan Python packages
    py_dir = repo_root / 'python-packages'
    if py_dir.exists():
        for pkg in sorted(py_dir.iterdir()):
            if pkg.is_dir():
                lines.append(
                    f'| `python-packages/{pkg.name}/` | (stub) | — | — | — | — |'
                )

    output_path.write_text('\n'.join(lines) + '\n')
    print(f"Repository map written to {output_path}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
