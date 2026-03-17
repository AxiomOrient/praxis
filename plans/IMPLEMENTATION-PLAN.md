# Praxis Phase Roadmap And Execution Plan

## Mission

Re-anchor Praxis around the final phased product shape so IA redesign becomes a first-order implementation concern instead of deferred cleanup.

## Canonical Authority

For this delivery path, authority is fixed in this order:

1. `plans/PRAXIS_FINAL_FORM_REFINEMENT.md`
2. `plans/PRAXIS_FINAL_SPEC.md`
3. `specs/03-SPEC.md`
4. `specs/04-RUNTIME-TARGET-PROFILES.md`

Implication:

- primary surfaces converge to `Discover`, `Library`, `Create`, `Benchmarks`
- utility surfaces converge to `Connections`, `Health`, `Settings`
- `Plan`, `Decks`, and `Agent Files` are contextual/detail surfaces, not peer top-level destinations
- Gemini remains integration-only until formally promoted

## Product Shape

Praxis final form is:

- `Library Plane`
- `Runtime Plane`
- `Evaluation Plane`

The current deterministic installer core is preserved as the Runtime Plane nucleus.

## Phase Map

### Phase 0. Canonical convergence

Goals:

- remove contract drift across plans, specs, README, examples, and desktop
- settle the canonical IA authority before more UI work lands
- finish terminology migration to `Agent Files`

Required outputs:

- one consistent phase roadmap
- one consistent navigation contract
- one consistent runtime-target contract

### Phase 1. Desktop shell convergence

Goals:

- reshape desktop around final navigation
- keep `plan` and `agent file editor` contextual
- map existing functionality into the new shell without inventing fake contracts

Required outputs:

- Discover surface over inspect/select/preview/apply
- Library surface over installed state and workspace outputs
- Health surface over doctor/runtime checks
- stable placeholders or scoped empty states for not-yet-implemented Create, Benchmarks, Connections

### Phase 2. Runtime Plane hardening

Goals:

- finish path invariants and runtime-target semantics
- add conformance tests around planner/reconciler/agent-file composition
- align desktop/backend schema rigorously

Required outputs:

- runtime target/profile invariants enforced
- contract tests for plan/apply/remove/sync/doctor
- desktop/backend schema conformance tests

### Phase 3. Library Plane

Goals:

- add SQLite-backed library metadata
- add artifact store and provenance model
- ingest source snapshots into a queryable local library

Required outputs:

- metadata DB schema
- filesystem artifact store layout
- internal sources for drafts/collections

### Phase 4. Evaluation Plane

Goals:

- add benchmark suite definitions and result persistence
- store candidate/current comparisons
- enable promotion evidence

Required outputs:

- benchmark schema
- result storage
- CLI/UI evidence surfaces

### Phase 5. Create system

Goals:

- add drafts, presets, preview tree, lineage, and promotion writer
- ensure authoring flows preserve repository contracts

Required outputs:

- create/import/fork/augment flows
- preview tree
- promotion/export path

### Phase 6. External execution and cooperative jobs

Goals:

- add one optional external LLM execution substrate without changing the local-first authority model
- implement persisted cooperative jobs with lease/reclaim semantics for long-running work
- route AI judge and human review through the same durable job model

Required outputs:

- one recommended provider bridge: `codex-runtime`
- persisted `jobs` execution path with `queued -> leased -> running -> terminal`
- worker lease, reclaim, heartbeat, cancel, and retry semantics
- desktop/CLI job queue and run detail surfaces

### Phase 7. Lineage UX and promotion review

Goals:

- make fork and augment lineage visible enough that promotion decisions are explainable
- show provenance, diffs, and evidence in one narrow review flow instead of spreading them across multiple panels
- keep lineage UX immutable, inspectable, and lightweight

Required outputs:

- lineage timeline or ancestry chain on skill detail
- explicit fork vs augment intent and parent version visibility
- promotion review surface that combines ancestry, changed files, benchmark evidence, and review state

## Current Execution Slice

This run executed the full P0→P5 convergence slice:

1. canonical/spec/README/desktop IA convergence
2. runtime workspace path invariants plus target-profile-aware validation
3. library metadata DB plus filesystem artifact store
4. evaluation suite/run persistence plus desktop evidence surfaces
5. create draft/preview/promotion flows against repository contracts

## Out Of Scope For This Run

- non-deterministic AI judge execution
- human review workflows and lease-based jobs
- advanced fork/augment lineage UX
- full authoring UI beyond draft/evidence summaries

## Recommended Next-Step Boundary

Use `codex-runtime` as the external LLM execution substrate when Praxis needs optional AI judge or assisted authoring turns.

Reasoning:

- it is a low-level session/runtime wrapper around the local `codex app-server`
- it already exposes reusable session, automation, and direct app-server layers
- it fits Praxis's local-first model because Praxis can keep SQLite/artifact store/jobs as authority and treat the LLM as one worker capability

Do not embed `AxiomRunner` into Praxis core.

Reasoning:

- it is a full goal-file autonomous runtime with its own run journal, control states, and operator surface
- its product boundary is broader than Praxis's missing gap
- integrating it into core would duplicate job state, run semantics, and review flow instead of simplifying them

Recommended use of `AxiomRunner`:

- keep it as a separate operator tool for dogfooding, batch validation, or external nightly automation
- if needed later, expose it as an export target or external executor profile, not as Praxis's internal job engine

## Verification Gates

1. `cargo test --workspace`
2. `npm run build` in `apps/praxis-desktop`
3. `rg -n 'Agent Files|My Skills|Decks|Guides' specs/00-HANDOFF.md specs/02-BLUEPRINT.md specs/07-UX-IA.md plans/PRAXIS_FINAL_FORM_REFINEMENT.md` shows one consistent IA story
4. desktop nav no longer exposes `Plan` or `Guides` as peer top-level destinations
5. runtime target defaults and validation are covered by core tests
6. desktop/backend schema conformance checks serialize current contract field names
7. benchmark suite bootstrap and persisted runs are covered by core tests
8. draft preview and repo-contract promotion flows are covered by core tests
9. external AI execution remains optional and cannot bypass persisted job/evidence state
10. lineage and promotion review surfaces stay derived from persisted provenance instead of ephemeral UI-only state
