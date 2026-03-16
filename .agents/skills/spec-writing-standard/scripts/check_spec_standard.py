#!/usr/bin/env python3
"""Validate that a spec follows the Praxis spec writing standard."""

from __future__ import annotations

import re
import sys
from pathlib import Path


TOP_LEVEL = [
    (1, re.compile(r"^## 1\. Problem Statement$")),
    (2, re.compile(r"^## 2\. Goals and Non-Goals$")),
    (3, re.compile(r"^## 3\. System Overview$")),
    (4, re.compile(r"^## 4\. Core Domain Model$")),
    (5, re.compile(r"^## 5\. Domain Contract$")),
    (6, re.compile(r"^## 6\. Configuration and Input Contract$")),
    (7, re.compile(r"^## 7\. Lifecycle or State Model$")),
    (8, re.compile(r"^## 8\. Primary Workflows and Reconciliation$")),
    (9, re.compile(r"^## 9\. Storage, Ownership, and Safety Boundaries$")),
    (10, re.compile(r"^## 10\. Execution or Interface Contract$")),
    (11, re.compile(r"^## 11\. External Integration Contract$")),
    (12, re.compile(r"^## 12\. Context Packaging and Prompt Inputs$")),
    (13, re.compile(r"^## 13\. Logging, Status, and Observability$")),
    (14, re.compile(r"^## 14\. Failure Model and Recovery Strategy$")),
    (15, re.compile(r"^## 15\. Safety, Boundaries, and Human Approval Policy$")),
    (16, re.compile(r"^## 16\. Reference Algorithms and Task Decomposition$")),
    (17, re.compile(r"^## 17\. Validation, Commands, and Success Criteria$")),
    (18, re.compile(r"^## 18\. Implementation Checklist and Change Control$")),
]

REQUIRED_SUBSECTIONS = [
    re.compile(r"^### 2\.1 Goals$"),
    re.compile(r"^### 2\.2 Non-Goals$"),
    re.compile(r"^### 3\.1 Main Components$"),
    re.compile(r"^### 3\.2 Abstraction Levels$"),
    re.compile(r"^### 3\.3 External Dependencies$"),
    re.compile(r"^### 3\.4 Project Structure and Key Paths$"),
    re.compile(r"^### 4\.1 Entities$"),
    re.compile(r"^### 4\.2 Stable Identifiers and Normalization Rules$"),
    re.compile(r"^### 12\.1 Required Context$"),
    re.compile(r"^### 12\.2 Task-Specific Context$"),
    re.compile(r"^### 15\.1 Always Do$"),
    re.compile(r"^### 15\.2 Ask First$"),
    re.compile(r"^### 15\.3 Never Do$"),
    re.compile(r"^### 17\.1 Commands$"),
    re.compile(r"^### 17\.2 Validation Matrix$"),
    re.compile(r"^### 17\.3 Acceptance and Conformance Gates$"),
    re.compile(r"^### 18\.1 Required for Conformance$"),
]


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_spec_standard.py <path/to/spec.md>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    if not path.exists():
        print(f"[FAIL] file not found: {path}")
        return 1

    lines = path.read_text(encoding="utf-8").splitlines()
    top_headers = [line.strip() for line in lines if line.startswith("## ")]
    sub_headers = [line.strip() for line in lines if line.startswith("### ")]

    errors: list[str] = []

    numbered_headers = []
    unexpected_non_numbered = []
    for header in top_headers:
        match = re.match(r"^## (\d+)\. ", header)
        if match:
            numbered_headers.append(int(match.group(1)))
            continue
        if header.startswith("## Appendix"):
            continue
        unexpected_non_numbered.append(header)

    if len(numbered_headers) != len(TOP_LEVEL):
        errors.append(
            f"expected exactly {len(TOP_LEVEL)} numbered top-level sections, found {len(numbered_headers)}"
        )

    expected_numbers = list(range(1, len(TOP_LEVEL) + 1))
    if numbered_headers != expected_numbers:
        errors.append(
            f"top-level numbering must be continuous {expected_numbers}, found {numbered_headers}"
        )

    if unexpected_non_numbered:
        errors.append(
            f"unexpected non-numbered top-level sections present: {unexpected_non_numbered}"
        )

    for index, pattern in TOP_LEVEL:
        if index - 1 >= len(top_headers):
            errors.append(f"missing top-level section {index}")
            continue
        header = top_headers[index - 1]
        if not pattern.match(header):
            errors.append(f"section {index} is out of place or incorrectly named: {header!r}")

    for pattern in REQUIRED_SUBSECTIONS:
        if not any(pattern.match(header) for header in sub_headers):
            errors.append(f"missing required subsection matching {pattern.pattern}")

    if not any(line.startswith("### 18.3") for line in lines):
        errors.append("section 18 must contain spec update triggers")

    if errors:
        print(f"[FAIL] {path}")
        for error in errors:
            print(f" - {error}")
        return 1

    print(f"[OK] {path}")
    print(" - top-level sections 1..18 present")
    print(" - AI execution policy and validation subsections present")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
