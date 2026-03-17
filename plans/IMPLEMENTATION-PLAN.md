# Public Runtime Contract Convergence Plan

Status: completed on 2026-03-17 after PRC-01 through PRC-06 landed and verification gates passed.

## Request

Remove Gemini from the public runtime-file contract, converge all public naming on `Agent Files`, and define one execution-ready path that can be implemented without reopening product-boundary debates.

This file supersedes the completed live `codex-runtime` validation plan as the active implementation plan.
The live validation slice was reviewed against code, tests, doctor output, runbook, and wrapper script, and no remaining implementation work was found in that older plan.

## Fixed Decisions

1. Public runtime-file targets are only `codex` and `claude`.
2. Gemini remains a prose-only future integration concept until a concrete runtime-file contract is formally adopted.
3. Public contracts must not expose Gemini runtime profiles, runtime-file slots, runtime paths, or install targets.
4. The canonical noun is `Agent Files`.
5. Public surface names must not use `guidance` or `guides`.
6. Backward-compatible read aliases are allowed only for persisted input compatibility where they reduce migration risk; they must not remain the canonical output shape.
7. Existing live `codex-runtime` validation artifacts remain in place and are not part of this change.

## Problem Statement

Praxis currently states in `README.md` and `specs/04-RUNTIME-TARGET-PROFILES.md` that Gemini is integration-only, while the public type system, workspace path model, CLI surface, desktop types, and bridge commands still expose Gemini runtime-file concepts.

Praxis also declares in `specs/00-HANDOFF.md` that `Guides` is retired language, while canonical and implementation surfaces still expose `guidance`, `guides`, and `guide` wording.

That creates two harmful states:

- the public contract advertises runtime-file capabilities that the product explicitly rejects
- the naming contract remains split between canonical docs and implementation surfaces

## Scope Contract

### In Scope

- canonical product docs and README
- public core model and serialized snapshot contract
- workspace path and runtime slot contract
- planner, doctor, and runtime validation behavior that depends on public runtime-file target types
- CLI command names and help text
- desktop bridge command names, API helpers, frontend types, i18n keys, HTML metadata, and visible copy
- test fixtures and snapshot assertions that currently encode Gemini runtime-file paths or legacy public nouns
- regression tests and grep/build gates that prove the contract convergence

### Out of Scope

- adding a new Gemini runtime-file contract
- redesigning the live `codex-runtime` executor slice
- broad create/evaluation product changes unrelated to runtime-file contract cleanup
- a compiled canonical spec artifact in this slice

## Design Summary

The change should split `runtime-file contract` from `future integration posture` at the type boundary rather than relying on runtime rejection.

### Target State

- `Agent`, `TargetProfile`, `AgentFileSlot`, `TargetPaths`, and desktop DTOs represent only supported runtime-file targets.
- No public command, DTO, or UI selector allows Gemini runtime-file selection.
- Source scanning and agent-file template discovery stop emitting Gemini runtime-file slots.
- Workspace path resolution and initialization stop creating Gemini runtime-file locations as part of the managed runtime contract.
- Canonical docs, README, CLI, desktop, and tests all use `Agent Files` terminology.

### Compatibility Rule

- Keep read-only compatibility aliases only where persisted user state may still contain legacy names, such as manifest field aliases.
- The preferred bounded exception is `guides` as a deserialize-only alias for persisted selection input.
- Do not keep legacy names in help text, command names, serialized snapshots, or frontend/public type definitions.
- Do not keep Gemini runtime-file fields as "reserved" public placeholders. A future Gemini adoption must add them back through an explicit contract change.

## Workstreams

### W1. Canonical Contract Convergence

- remove Gemini runtime-file targets from canonical docs and README surface definitions
- replace remaining `guide`/`guidance`/`guides` product nouns with `agent files` or `agent file templates`
- make the docs state that future Gemini support requires a fresh contract adoption change

### W2. Public Type-System Surgery

- remove Gemini runtime-file variants from public Rust enums and DTOs
- remove Gemini runtime-file fields from workspace snapshots and desktop types
- delete runtime validation branches that only exist because unsupported Gemini variants are still representable

### W3. Runtime and Scan Boundary Cleanup

- stop creating Gemini runtime skill roots and runtime-file paths inside managed workspace setup
- stop resolving Gemini agent-file slots inside managed composition
- stop discovering or recommending Gemini runtime-file templates from scanned sources
- update test helpers and fixture constructors that currently fabricate Gemini runtime-file paths so the suite reflects the new contract rather than legacy placeholders

### W4. Surface Renaming

- rename CLI `Guidance` to `AgentFiles`
- rename desktop bridge/API commands away from `guidance`/`guidance_write`
- rename i18n keys, HTML metadata, source descriptions, and visible copy so public surface vocabulary is internally consistent

### W5. Regression and Migration Gates

- add tests that legacy manifest input still deserializes where intended
- add tests that public snapshots no longer serialize Gemini runtime-file fields
- add grep/build gates that fail on public `guidance`/`guides` regressions outside allowed reference URLs or compatibility aliases

## Verification Gates

1. `cargo test --workspace`
2. `npm run build` in `apps/praxis-desktop`
3. `rg -n 'guide outputs|guide merge|Guidance\\b|guidance_write|guidance\\(|nav\\.guides|hero\\.guides|common\\.guides|plan\\.guides|guides\\.slot|guides\\.noPath' README.md specs crates apps --glob '!specs/99-REFERENCES.md'`
4. `rg -n 'Agent::Gemini|GeminiNative|CodexGeminiSharedOpenStandard|GeminiUserRoot|GeminiProjectRoot|gemini_project_root|gemini_skills' crates apps`
5. serialization tests prove current public snapshot keys no longer include Gemini runtime-file fields

## Done Condition

The slice is complete when:

- a reader can inspect README, specs, CLI help, desktop UI, and snapshot types without seeing Gemini presented as a managed runtime-file target
- a reader can inspect the same surfaces without encountering `Guidance` or `Guides` as canonical product nouns
- persisted legacy manifest input still reads where explicitly supported
- all listed verification gates pass

## Execution Order

1. update canonical docs and README first so the implementation change lands against the intended contract
2. remove Gemini runtime-file exposure from the public data model and workspace contract
3. converge CLI, Tauri bridge, desktop API, frontend types, and UI copy on the new contract
4. add regression coverage and run gates

## Self-Review Checklist

- Scope bounded to runtime-file contract and naming convergence: yes
- Product decision resolved before tasking: yes
- Verification externally observable: yes
- Touched surface covers docs, public types, bridge/UI naming, metadata copy, and fixtures: yes
- Existing live executor plan preserved instead of overwritten: yes
- Hidden blocker requiring user clarification: no
