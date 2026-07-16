#!/usr/bin/env python3
"""Requirement integrity validator (DOC-001).

Validates:
1. Every normative REQ ID heading has at least one VERIF ID in the CSV
2. Every VERIF ID in the CSV references a REQ ID that exists in the SRS
3. No duplicate VERIF IDs
4. No duplicate normative REQ ID headings in the SRS
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

# Only match heading-style normative definitions: "## REQ-ID — Name"
NORMATIVE_HEADING_RE = re.compile(
    r"^##\s+([A-Z]+-\d{3})\s+—\s+(.+?)\s*$",
    re.MULTILINE,
)
# Match any REQ ID pattern for cross-referencing
REQ_ID_RE = re.compile(r'\b([A-Z]+-\d{3})\b')
VERIF_ID_RE = re.compile(r'\b([A-Z]+-\d{3}-V\d+)\b')

# All Unicode categories that start with 'C' (control/format/private-use/surrogate)
# We check for invisible characters in IDs by scanning raw text around matches
def _has_invisible_chars(s: str) -> bool:
    for ch in s:
        cat = unicodedata.category(ch)
        if cat.startswith('C'):
            return True
    return False


def extract_normative_headings(text: str) -> dict[str, str]:
    """Extract REQ IDs and names from normative heading lines only."""
    headings = {}
    for match in NORMATIVE_HEADING_RE.finditer(text):
        headings[match.group(1)] = match.group(2).strip()
    return headings


def extract_all_req_ids(text: str) -> set[str]:
    """Extract all REQ ID occurrences (headings and references)."""
    return set(REQ_ID_RE.findall(text))


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
            if d.is_dir() and VERIF_ID_RE.match(d.name):
                verif_dirs.add(d.name)
    return verif_dirs


def check_identifier(identifier: str, location: str) -> list[str]:
    """Check for invisible/control characters inside REQ and VERIF IDs."""
    issues: list[str] = []
    for ch in identifier:
        cat = unicodedata.category(ch)
        if cat.startswith('C'):
            issues.append(
                f"Invisible character U+{ord(ch):04X} in {location}: {identifier!r}"
            )
    return issues


def check_duplicate_normative_headings(text: str) -> list[str]:
    """Check for duplicate normative REQ ID headings."""
    issues: list[str] = []
    seen: dict[str, int] = {}
    for match in NORMATIVE_HEADING_RE.finditer(text):
        req_id = match.group(1)
        line_no = text[:match.start()].count('\n') + 1
        if req_id in seen:
            issues.append(
                f"Duplicate normative heading '{req_id}' at line {line_no} "
                f"(previously defined at line {seen[req_id]})"
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
    srs_headings = extract_normative_headings(srs_text)
    srs_all_ids = extract_all_req_ids(srs_text)
    matrix_rows = parse_verification_matrix(matrix_path)

    matrix_req_ids: set[str] = set()
    matrix_verif_ids: set[str] = set()

    for row in matrix_rows:
        req_id = row.get('Req ID', '').strip()
        verif_id = row.get('Verif ID', '').strip()
        req_name = row.get('Req Name', '').strip()

        if req_id:
            matrix_req_ids.add(req_id)
            if control_issues := check_identifier(req_id, "Matrix REQ ID"):
                errors.extend(control_issues)
            
            if req_id in srs_headings:
                if req_name != srs_headings[req_id]:
                    errors.append(f"Name mismatch for {req_id}: Matrix says {req_name!r}, SRS says {srs_headings[req_id]!r}")

        if verif_id:
            if verif_id in matrix_verif_ids:
                errors.append(f"Duplicate VERIF ID in matrix: {verif_id}")
            matrix_verif_ids.add(verif_id)
            if control_issues := check_identifier(verif_id, "Matrix VERIF ID"):
                errors.extend(control_issues)

    # Check invisible/control characters in the SRS headings
    for req_id, req_name in srs_headings.items():
        if control_issues := check_identifier(req_id, "SRS REQ ID"):
            errors.extend(control_issues)
        if control_issues := check_identifier(req_name, "SRS REQ Name"):
            errors.extend(control_issues)

    # Check duplicate normative headings
    dup_issues = check_duplicate_normative_headings(srs_text)
    errors.extend(dup_issues)

    # Check 1: Every matrix REQ ID exists as a normative heading in the SRS
    for req_id in sorted(matrix_req_ids):
        if req_id not in srs_headings:
            errors.append(f"REQ ID '{req_id}' in matrix has no corresponding normative heading in SRS")

    # Check 2: Every normative heading has at least one verification
    for req_id in sorted(srs_headings):
        if req_id not in matrix_req_ids:
            errors.append(
                f"Normative REQ ID '{req_id}' has no verification row in matrix"
            )

    # Check 3: Each automated VERIF ID must have a test directory
    verif_test_dirs = find_verif_test_dirs(Path(args.verification_dir))
    for verif_id in sorted(matrix_verif_ids):
        if verif_id not in verif_test_dirs:
            verif_type = next(
                (r.get('Verif Type', '') for r in matrix_rows if r.get('Verif ID', '') == verif_id),
                ''
            )
            if verif_type in ('AnalysisArtifact', 'ManualVerification'):
                warnings.append(
                    f"VERIF ID '{verif_id}' is {verif_type} - no test directory expected"
                )
            else:
                errors.append(
                    f"VERIF ID '{verif_id}' ({verif_type}) has no test directory in verification/tests/"
                )

    # Report
    print(f"\n=== Requirement Integrity Report ===")
    print(f"Normative headings: {len(srs_headings)}")
    print(f"All SRS REQ IDs:    {len(srs_all_ids)}")
    print(f"Matrix REQ IDs:     {len(matrix_req_ids)}")
    print(f"Matrix VERIF IDs:   {len(matrix_verif_ids)}")
    print(f"Test directories:   {len(verif_test_dirs)}")
    print(f"Errors:             {len(errors)}")
    print(f"Warnings:           {len(warnings)}")

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
