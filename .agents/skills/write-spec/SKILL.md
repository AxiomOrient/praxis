---
name: write-spec
description: Draft, refactor, or audit a contract-style specification. Use when Codex must turn loose product notes, PRDs, architecture memos, workflow docs, or implementation plans into a numbered `SPEC.md`-class document with explicit boundaries, typed entities, contracts, state/lifecycle rules, failure and recovery behavior, validation matrices, and a definition-of-done checklist.
---

# Write Spec

## Purpose

Produce a specification that reads like a contract, not a narrative.
Default to the standard top-level section order and only rename section titles when the exact service wording would be misleading for the target domain.

## Workflow

1. Read [references/spec-writing-guide.md](references/spec-writing-guide.md).
2. Start from [assets/spec-template.md](assets/spec-template.md).
3. Define the product or system boundary before writing section 3 or later.
4. Name stable entities, ids, states, and outputs before filling detailed rules.
5. Fill sections in order. Do not skip a top-level section silently.
6. If a section is genuinely out of scope, state that explicitly inside the section instead of deleting it.
7. Run `python3 scripts/check_spec.py <path/to/spec.md>` before finishing.
8. When the target is meant to mirror a service-spec shape closely, also run `python3 scripts/check_spec.py --mode strict <path/to/spec.md>`.

## Writing Rules

- Keep the top-level numbered structure `1` through `18`.
- Prefer contract language: `must`, `should`, `may`, `out of scope`, `required`, `optional`.
- Separate observed facts from inferred design choices.
- Define stable nouns once and reuse them exactly.
- State normalization, ordering, determinism, and recovery rules explicitly.
- Include failure classes, operator or user-visible recovery paths, and a validation matrix.
- End with a concrete implementation checklist rather than vague future work.

## Adaptation Rules

- Keep sections `1` through `4` exactly aligned with contract intent: problem, goals, overview, domain model.
- Treat sections `5` through `12` as domain contracts. Rename them only enough to fit the actual system.
- Use section `7` for state machine or lifecycle state model. Do not leave lifecycle implicit.
- Use section `14` for failure and recovery, not only a list of errors.
- Use section `17` for validation evidence and section `18` for definition of done.
- Use strict mode only when the output should stay close to service-spec headings instead of an adapted product-spec variant.

## Resources

- Use [references/spec-writing-guide.md](references/spec-writing-guide.md) for the exact meaning of "Contract-style".
- Use [assets/spec-template.md](assets/spec-template.md) as the scaffold for new specs.
- Use `scripts/check_spec.py` to validate heading coverage and section continuity.
