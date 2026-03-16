# Praxis Handoff and Canonical Spec Package

This package is the **handoff boundary** for the next agent.
It assumes the earlier rename, management-plane framing, and agent-file expansion have already been accepted.

The purpose of this package is not to preserve implementation trivia.
Its purpose is to preserve the **final product truth** and the **canonical specifications** required to finish Praxis without re-litigating solved decisions.

The package is intentionally split instead of monolithic:

- `03-SPEC.md` is the canonical contract document and follows the section-oriented style used by OpenAI Symphony's `SPEC.md`, adapted for a product specification instead of a service runner.
- the surrounding files exist to keep boundary-specific contracts inspectable without overloading one file
- if a boundary can be defined cleanly in a subordinate file, `03-SPEC.md` should state the canonical rule and defer details rather than duplicating them

## What this package contains

- `00-HANDOFF.md` — compressed executive handoff
- `01-PHILOSOPHY.md` — product philosophy and non-negotiable principles
- `02-BLUEPRINT.md` — final product shape and surfaces
- `03-SPEC.md` — canonical, Symphony-style product specification
- `04-RUNTIME-TARGET-PROFILES.md` — Codex / Claude Code / Gemini CLI target profiles
- `05-REPOSITORY-CONTRACTS.md` — source repository contracts and optional Praxis metadata
- `06-CREATION-SYSTEM.md` — skill / deck / agent-file creation system specification
- `07-UX-IA.md` — information architecture and screen system
- `08-DISTRIBUTION.md` — release shape and packaging boundaries
- `99-REFERENCES.md` — primary references used for this package

## Reading order

1. `00-HANDOFF.md`
2. `01-PHILOSOPHY.md`
3. `02-BLUEPRINT.md`
4. `03-SPEC.md`
5. `04-RUNTIME-TARGET-PROFILES.md`
6. `05-REPOSITORY-CONTRACTS.md`
7. `06-CREATION-SYSTEM.md`
8. `07-UX-IA.md`
9. `08-DISTRIBUTION.md`
10. `99-REFERENCES.md`

## Canonical rule

If a future document conflicts with `03-SPEC.md`, `03-SPEC.md` wins.
If a surface-level decision conflicts with `01-PHILOSOPHY.md`, the philosophy wins.
If a planning document conflicts with anything in `specs/`, the `specs/` package wins.

## Scope boundary

This package defines the final product and its contracts.
It intentionally does **not** include implementation tasks, issue breakdowns, file-level coding instructions, or test plans.
