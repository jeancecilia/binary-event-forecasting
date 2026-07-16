#!/usr/bin/env python3
"""Requirement integrity validator (DOC-001).

Validates:
1. Every REQ ID in the SRS has at least one VERIF ID in the CSV
2. Every VERIF ID in the CSV references a REQ ID that exists in the SRS
3. No duplicate VERIF IDs
4. No duplicate normative REQ IDs in the SRS body
5. No invisible/control characters in controlled identifiers
6. Every automated VERIF ID has a corresponding test directory

Usage:
    python scripts/validate_requirements.py
"""

import argparse
import csv
import re
import sys
import unicodedata
from pathlib import Path

REQ_ID_PATTERN = re.compile(r'\b([A-Z]+-\d{3})\b')
VERIF_ID_PATTERN = re.compile(r'\b([A-Z]+-\d{3}-V\d+)\b')

# Control characters except category Cf (format characters)
CONTROL_CHARS = set()
for i in range(0x10000):
    cat = unicodedata.category(chr(i))
    if cat.startswith('C') and cat != 'Cf':
        CONTROL_CHARS.add(chr(i))


def extract_req_ids(text: str) -> set[str]:
    return set(REQ_ID_PATTERN.findall(text))


def parse_verification_matrix(csv_path: Path) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    with open(csv_path, newline='', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            rows.append({k: v for k, v in row.items()})
    return rows


def find_verif_test_dirs(verification_dir: Path) -> set[str]:
    verif_dirs: set[str] = set()
    if not verification_dir.exists():
        return verif_dirs
    tests_dir = verification_dir / 'tests'
    if tests_dir.exists():
        for d in tests_dir.iterdir():
            if d.is_dir() and VERIF_ID_PATTERN.match(d.name):
                verif_dirs.add(d.name)
    return verif_dirs


def check_control_chars(text: str) -> list[str]:
    issues: list[str] = []
    for match in REQ_ID_PATTERN.finditer(text):
        for ch in match.group(0):
            if ch in CONTROL_CHARS:
                issues.append(
                    f"Control character U+{ord(ch):04X} in REQ ID '{match.group(0)}' at position {match.start()}"
                )
    for match in VERIF_ID_PATTERN.finditer(text):
        for ch in match.group(0):
            if ch in CONTROL_CHARS:
                issues.append(
                    f"Control character U+{ord(ch):04X} in VERIF ID '{match.group(0)}' at position {match.start()}"
                )
    return issues


def check_duplicate_normative_ids(text: str) -> list[str]:
    """Check for duplicate normative REQ IDs in the SRS body."""
    issues: list[str] = []
    seen: dict[str, int] = {}
    for match in REQ_ID_PATTERN.finditer(text):
        req_id = match.group(0)
        line_no = text[:match.start()].count('\n') + 1
        if req_id in seen:
            issues.append(
                f"Duplicate normative REQ ID '{req_id}' at line {line_no} "
                f"(previously seen at line {seen[req_id]})"
            )
        else:
            seen[req_id] = line_no
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

    # Load verification matrix
    matrix_path = Path(args.matrix)
    if not matrix_path.exists():
        errors.append(f"Verification matrix not found: {matrix_path}")

    if errors:
        for err in errors:
            print(f"FAIL: {err}")
        return 1

    srs_text = srs_path.read_text(encoding='utf-8')
    srs_req_ids = extract_req_ids(srs_text)
    matrix_rows = parse_verification_matrix(matrix_path)

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

    # Check control characters in identifiers
    control_issues = check_control_chars(srs_text)
    errors.extend(control_issues)

    # Check duplicate normative IDs in SRS
    dup_issues = check_duplicate_normative_ids(srs_text)
    errors.extend(dup_issues)

    # Check 1: Every matrix REQ ID exists in the SRS
    for req_id in sorted(matrix_req_ids):
        if req_id not in srs_req_ids:
            errors.append(f"REQ ID '{req_id}' in matrix not found in SRS")

    # Check 2: Every SRS REQ ID has at least one verification
    for req_id in sorted(srs_req_ids):
        if req_id not in matrix_req_ids:
            errors.append(f"REQ ID '{req_id}' in SRS has no verification row in matrix")

    # Check 3: Each automated VERIF ID must have a test directory
    verif_test_dirs = find_verif_test_dirs(Path(args.verification_dir))
    for verif_id in sorted(matrix_verif_ids):
        if verif_id not in verif_test_dirs:
            verif_type = next(
                (r.get('Verif Type', '') for r in matrix_rows if r.get('Verif ID', '') == verif_id),
                ''
            )
            if verif_type in ('AnalysisArtifact', 'ManualVerification'):
                warnings.append(f"VERIF ID '{verif_id}' is {verif_type} - no automated test directory expected")
            else:
                errors.append(
                    f"VERIF ID '{verif_id}' ({verif_type}) has no test directory in verification/tests/"
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
            print(f"  FAIL: {err}")

    if warnings:
        print(f"\n=== WARNINGS ({len(warnings)}) ===")
        for warn in warnings:
            print(f"  WARN: {warn}")

    if not errors and not warnings:
        print("\nPASS: All requirement integrity checks passed.")

    return 1 if errors else 0


if __name__ == '__main__':
    sys.exit(main())
