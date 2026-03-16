#!/usr/bin/env python3
"""Validate that a spec follows the standard contract shape."""

from __future__ import annotations

import re
import sys
from pathlib import Path
import argparse


ADAPTED_TOP_LEVEL = [
    (1, re.compile(r"^## 1\. Problem Statement$")),
    (2, re.compile(r"^## 2\. Goals and Non-Goals$")),
    (3, re.compile(r"^## 3\. System Overview$")),
    (4, re.compile(r"^## 4\. Core Domain Model$")),
    (5, re.compile(r"^## 5\. .+$")),
    (6, re.compile(r"^## 6\. .+$")),
    (7, re.compile(r"^## 7\. .+$")),
    (8, re.compile(r"^## 8\. .+$")),
    (9, re.compile(r"^## 9\. .+$")),
    (10, re.compile(r"^## 10\. .+$")),
    (11, re.compile(r"^## 11\. .+$")),
    (12, re.compile(r"^## 12\. .+$")),
    (13, re.compile(r"^## 13\. .+$")),
    (14, re.compile(r"^## 14\. .+$")),
    (15, re.compile(r"^## 15\. .+$")),
    (16, re.compile(r"^## 16\. .+$")),
    (17, re.compile(r"^## 17\. .+$")),
    (18, re.compile(r"^## 18\. .+$")),
]

STRICT_TOP_LEVEL = [
    (1, re.compile(r"^## 1\. Problem Statement$")),
    (2, re.compile(r"^## 2\. Goals and Non-Goals$")),
    (3, re.compile(r"^## 3\. System Overview$")),
    (4, re.compile(r"^## 4\. Core Domain Model$")),
    (5, re.compile(r"^## 5\. Workflow Specification \(Repository Contract\)$")),
    (6, re.compile(r"^## 6\. Configuration Specification$")),
    (7, re.compile(r"^## 7\. Orchestration State Machine$")),
    (8, re.compile(r"^## 8\. Polling, Scheduling, and Reconciliation$")),
    (9, re.compile(r"^## 9\. Workspace Management and Safety$")),
    (10, re.compile(r"^## 10\. Agent Runner Protocol \(Coding Agent Integration\)$")),
    (11, re.compile(r"^## 11\. Issue Tracker Integration Contract .+$")),
    (12, re.compile(r"^## 12\. Prompt Construction and Context Assembly$")),
    (13, re.compile(r"^## 13\. Logging, Status, and Observability$")),
    (14, re.compile(r"^## 14\. Failure Model and Recovery Strategy$")),
    (15, re.compile(r"^## 15\. Security and Operational Safety$")),
    (16, re.compile(r"^## 16\. Reference Algorithms .*$")),
    (17, re.compile(r"^## 17\. Test and Validation Matrix$")),
    (18, re.compile(r"^## 18\. Implementation Checklist .*$")),
]

REQUIRED_SUBSECTIONS = [
    re.compile(r"^### 2\.1 Goals$"),
    re.compile(r"^### 2\.2 Non-Goals$"),
    re.compile(r"^### 3\.1 Main Components$"),
    re.compile(r"^### 3\.2 Abstraction Levels$"),
    re.compile(r"^### 3\.3 External Dependencies$"),
    re.compile(r"^### 4\.1 Entities$"),
    re.compile(r"^### 4\.2 Stable Identifiers and Normalization Rules$"),
]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate that a spec follows the standard contract shape."
    )
    parser.add_argument(
        "--mode",
        choices=("adapted", "strict"),
        default="adapted",
        help="Validation mode. Use strict to stay close to service-spec headings.",
    )
    parser.add_argument("path", help="Path to the spec markdown file.")
    return parser


def main() -> int:
    args = build_parser().parse_args()
    path = Path(args.path)
    if not path.exists():
        print(f"[FAIL] file not found: {path}", file=sys.stderr)
        return 1

    lines = path.read_text(encoding="utf-8").splitlines()
    top_headers = [line.strip() for line in lines if line.startswith("## ")]
    sub_headers = [line.strip() for line in lines if line.startswith("### ")]

    errors: list[str] = []
    top_level = STRICT_TOP_LEVEL if args.mode == "strict" else ADAPTED_TOP_LEVEL

    numbered_headers = []
    appendix_headers = []
    unexpected_non_numbered = []
    for header in top_headers:
        match = re.match(r"^## (\d+)\. ", header)
        if match:
            numbered_headers.append(int(match.group(1)))
            continue
        if header.startswith("## Appendix"):
            appendix_headers.append(header)
            continue
        unexpected_non_numbered.append(header)

    if len(numbered_headers) != len(top_level):
        errors.append(
            f"expected exactly {len(top_level)} numbered top-level sections, found {len(numbered_headers)}"
        )

    expected_numbers = list(range(1, len(top_level) + 1))
    if numbered_headers != expected_numbers:
        errors.append(
            f"top-level section numbering must be continuous {expected_numbers}, found {numbered_headers}"
        )
    if unexpected_non_numbered:
        errors.append(
            f"unexpected non-numbered top-level sections present: {unexpected_non_numbered}"
        )

    for index, pattern in top_level:
        if index - 1 >= len(top_headers):
            errors.append(f"missing top-level section {index}")
            continue
        header = top_headers[index - 1]
        if not pattern.match(header):
            errors.append(
                f"section {index} is out of place or incorrectly named: {header!r}"
            )

    for pattern in REQUIRED_SUBSECTIONS:
        if not any(pattern.match(header) for header in sub_headers):
            errors.append(f"missing required subsection matching {pattern.pattern}")

    if not any(line.startswith("### 17.") for line in lines):
        errors.append("section 17 must contain at least one validation subsection")
    if not any(line.startswith("### 18.1") for line in lines):
        errors.append("section 18 must contain a conformance checklist subsection")

    if errors:
        print(f"[FAIL] {path} (mode={args.mode})")
        for error in errors:
            print(f" - {error}")
        return 1

    print(f"[OK] {path} (mode={args.mode})")
    print(" - top-level sections 1..18 present")
    print(" - required core subsections present")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
