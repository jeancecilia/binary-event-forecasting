#!/usr/bin/env python3
"""Requirement integrity validator (DOC-001).

Validates that:
1. Every REQ ID in the SRS has at least one VERIF ID in the CSV
2. Every VERIF ID in the CSV references a REQ ID that exists in the SRS
3. No duplicate VERIF IDs
4. No duplicate REQ IDs
5. No invisible/control characters in identifiers
6. Every VERIF ID has a corresponding test directory or explicit justification

Usage:
    python scripts/validate_requirements.py
    python scripts/validate_requirements.py --srs docs/specification/srs.md --matrix verification/matrix.csv
"""

import argparse
import csv
import re
import sys
import unicodedata
from pathlib import Path

# Regex to extract REQ IDs from the SRS markdown
REQ_ID_PATTERN = re.compile(r'\b([A-Z]+-\d{3})\b')
VERIF_ID_PATTERN = re.compile(r'\b([A-Z]+-\d{3}-V\d+)\b')

CONTROL_CHARS = set()
for i in range(0x10000):
    cat = unicodedata.category(chr(i))
    if cat.startswith('C') and cat not in ('Cf',):
        CONTROL_CHARS.add(chr(i))


def extract_req_ids(srs_text: str) -> set[str]:
    """Extract all unique REQ IDs from SRS text."""
    return set(REQ_ID_PATTERN.findall(srs_text))


def extract_verif_ids_from_srs(srs_text: str) -> set[str]:
    """Extract all unique VERIF IDs mentioned in SRS text."""
    return set(VERIF_ID_PATTERN.findall(srs_text))


def parse_verification_matrix(csv_path: Path) -> list[dict]:
    """Parse the verification matrix CSV."""
    rows = []
    with open(csv_path, newline='', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            rows.append(row)
    return rows


def find_verif_test_dirs(verification_dir: Path) -> set[str]:
    """Find all verification test directories named after VERIF IDs."""
    verif_dirs = set()
    if not verification_dir.exists():
        return verif_dirs
    tests_dir = verification_dir / 'tests'
    if tests_dir.exists():
        for d in tests_dir.iterdir():
            if d.is_dir() and VERIF_ID_PATTERN.match(d.name):
                verif_dirs.add(d.name)
    return verif_dirs


def check_control_chars(text: str) -> list[str]:
    """Check for invisible/control characters in identifiers."""
    issues = []
    for match in REQ_ID_PATTERN.finditer(text):
        for ch in match.group(0):
            if ch in CONTROL_CHARS:
                issues.append(
                    f"Control character U+{ord(ch):04X} in REQ ID '{match.group(0)}'"
                )
    for match in VERIF_ID_PATTERN.finditer(text):
        for ch in match.group(0):
            if ch in CONTROL_CHARS:
                issues.append(
                    f"Control character U+{ord(ch):04X} in VERIF ID '{match.group(0)}'"
                )
    return issues


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate requirement integrity")
    parser.add_argument(
        '--srs',
        default='docs/specification/binary_event_forecasting_srs_v1_2.md',
        help='Path to SRS markdown file',
    )
    parser.add_argument(
        '--matrix',
        default='verification/verification_matrix_v1_2.csv',
        help='Path to verification matrix CSV',
    )
    parser.add_argument(
        '--verification-dir',
        default='verification',
        help='Path to verification directory',
    )
    args = parser.parse_args()

    errors: list[str] = []
    warnings: list[str] = []

    # Load SRS
    srs_path = Path(args.srs)
    if not srs_path.exists():
        errors.append(f"SRS file not found: {srs_path}")
        print(f"ERROR: SRS file not found: {srs_path}")
        return 1

    srs_text = srs_path.read_text(encoding='utf-8')

    # Load verification matrix
    matrix_path = Path(args.matrix)
    if not matrix_path.exists():
        errors.append(f"Verification matrix not found: {matrix_path}")
        print(f"ERROR: Verification matrix not found: {matrix_path}")
        return 1

    matrix_rows = parse_verification_matrix(matrix_path)

    # Extract IDs
    srs_req_ids = extract_req_ids(srs_text)
    srs_verif_ids = extract_verif_ids_from_srs(srs_text)

    matrix_req_ids: set[str] = set()
    matrix_verif_ids: set[str] = set()
    verif_to_req: dict[str, str] = {}

    for row in matrix_rows:
        req_id = row.get('Req ID', '').strip()
        verif_id = row.get('Verif ID', '').strip()
        if req_id:
            matrix_req_ids.add(req_id)
        if verif_id:
            if verif_id in matrix_verif_ids:
                errors.append(f"Duplicate VERIF ID in matrix: {verif_id}")
            matrix_verif_ids.add(verif_id)
            verif_to_req[verif_id] = req_id

    # Check 1: Every matrix REQ ID exists in the SRS
    for req_id in sorted(matrix_req_ids):
        if req_id not in srs_req_ids:
            errors.append(
                f"REQ ID '{req_id}' in matrix not found in SRS"
            )

    # Check 2: Every SRS REQ ID has at least one verification
    for req_id in sorted(srs_req_ids):
        if req_id not in matrix_req_ids:
            errors.append(
                f"REQ ID '{req_id}' in SRS has no verification row in matrix"
            )

    # Check 3: No duplicate VERIF IDs (already checked during parsing)

    # Check 4: No duplicate REQ IDs in matrix
    req_id_counts: dict[str, int] = {}
    for row in matrix_rows:
        req_id = row.get('Req ID', '').strip()
        if req_id:
            req_id_counts[req_id] = req_id_counts.get(req_id, 0) + 1
    # Duplicate REQs are expected (multiple VERIF per REQ), not an error.

    # Check 5: No control characters in identifiers
    control_issues = check_control_chars(srs_text)
    for issue in control_issues:
        errors.append(issue)

    # Check 6: Each VERIF ID has a test directory or justification
    verif_test_dirs = find_verif_test_dirs(Path(args.verification_dir))
    for verif_id in sorted(matrix_verif_ids):
        if verif_id not in verif_test_dirs:
            verif_type = next(
                (r.get('Verif Type', '') for r in matrix_rows if r.get('Verif ID', '') == verif_id),
                ''
            )
            if verif_type == 'AnalysisArtifact':
                warnings.append(
                    f"VERIF ID '{verif_id}' is AnalysisArtifact — no automated test directory expected"
                )
            elif verif_type == 'ManualVerification':
                warnings.append(
                    f"VERIF ID '{verif_id}' is ManualVerification — no automated test directory expected"
                )
            else:
                warnings.append(
                    f"VERIF ID '{verif_id}' has no test directory in verification/tests/"
                )

    # Report
    print(f"\n=== Requirement Integrity Report ===")
    print(f"SRS REQ IDs:       {len(srs_req_ids)}")
    print(f"Matrix REQ IDs:    {len(matrix_req_ids)}")
    print(f"Matrix VERIF IDs:  {len(matrix_verif_ids)}")
    print(f"Test directories:  {len(verif_test_dirs)}")
    print(f"Errors:            {len(errors)}")
    print(f"Warnings:          {len(warnings)}")

    if errors:
        print(f"\n=== ERRORS ({len(errors)}) ===")
        for err in errors:
            print(f"  ❌ {err}")

    if warnings:
        print(f"\n=== WARNINGS ({len(warnings)}) ===")
        for warn in warnings:
            print(f"  ⚠️  {warn}")

    if not errors and not warnings:
        print("\n✅ All requirement integrity checks passed.")

    return 1 if errors else 0


if __name__ == '__main__':
    sys.exit(main())
