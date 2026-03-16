---
name: spec-writing-standard
description: Write or upgrade an AI-facing specification using the Praxis spec writing standard. Use when a repo needs a precise, implementation-ready spec with explicit goals, contracts, states, failures, commands, validation gates, project structure, and Always/Ask First/Never boundaries. Do not use for lightweight briefs, changelogs, or loose brainstorm notes.
---

# Spec Writing Standard

## Purpose

Produce a specification that is safe and specific enough for autonomous coding work.
This skill combines contract-style spec structure with AI execution rules.

## Workflow

1. Read [plans/PRAXIS_SPEC_WRITING_STANDARD.md](../../../plans/PRAXIS_SPEC_WRITING_STANDARD.md).
2. Decide whether the task needs a full 18-section spec or a smaller scoped artifact.
3. For a full implementation spec, start from [assets/spec-writing-standard-template.md](assets/spec-writing-standard-template.md).
4. Ground the draft in current repository evidence before making design decisions.
5. Make the six operational areas explicit: commands, testing, project structure, code style, git workflow, boundaries.
6. Include `Always Do`, `Ask First`, and `Never Do` policy in section 15.
7. Make section 12 explicit about context packaging and task slicing for AI work.
8. Make section 17 explicit about commands, validation, and success gates.
9. Run `python3 scripts/check_spec_standard.py <path/to/spec.md>` before finishing.

## Writing Rules

- Prefer direct obligations over vague prose.
- Separate observed facts from chosen design.
- Do not leave state, failure, recovery, or migration implicit.
- Do not claim a spec is complete unless commands and validation gates are present.
- Keep terms stable across the document.
- If the spec is AI-facing, it must be possible to hand only the relevant sections to the agent for a bounded task.

## When To Use

- Converting loose notes or PRDs into a working implementation spec.
- Upgrading an existing spec so it can guide AI coding agents safely.
- Defining repository standards for execution, planning, validation, and handoff.

## When Not To Use

- One-off idea capture.
- Product marketing copy.
- Roadmaps or changelogs.
- Tiny fixes that only need a short task note.

## Resources

- Canonical standard: [plans/PRAXIS_SPEC_WRITING_STANDARD.md](../../../plans/PRAXIS_SPEC_WRITING_STANDARD.md)
- Template: [assets/spec-writing-standard-template.md](assets/spec-writing-standard-template.md)
- Validator: `scripts/check_spec_standard.py`
