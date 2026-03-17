# Praxis spec-method comparison vs OpenAI Symphony

## 0. Scope

This document compares two **specification methods**, not two products:

- **OpenAI Symphony** — a single-file, service-oriented canonical `SPEC.md`
- **Praxis** — a split product-spec package centered on `specs/03-SPEC.md` plus boundary-specific companion specs

The comparison is grounded in the current committed repositories.

## 1. Snapshot: Symphony method

Observed structure:
- one canonical `SPEC.md`
- strong monolithic contract density
- service-oriented 18-ish contract flow
- explicit file contract (`WORKFLOW.md`)
- typed config schema and defaults
- dynamic reload semantics
- orchestration state machine
- polling/scheduling/reconciliation rules
- explicit error surface
- validation/preflight semantics
- direct “implement according to this spec” portability

Operational character:
- optimized for immediate implementation by an agent or engineer
- optimized for low ambiguity in one bounded service
- optimized for “single document handoff”

## 2. Snapshot: current Praxis method

Observed structure:
- split spec package under `specs/`
- `specs/03-SPEC.md` declared canonical
- separate documents for:
  - philosophy
  - blueprint
  - runtime targets
  - repository contracts
  - creation system
  - UX/IA
  - distribution
  - references
- additional non-package documents under `docs/` and `plans/`

Operational character:
- optimized for decomposition of a broader product
- optimized for preserving boundary-specific thinking
- less optimized for one-shot implementation handoff unless the reader knows the precedence rules well

## 3. Side-by-side comparison

| Axis | Symphony method | Praxis method | Who wins |
|---|---|---|---|
| Canonicality clarity | One file is obviously canonical | `03-SPEC.md` is canonical, but surrounding docs still make conflicting claims | Symphony |
| Boundary decomposition | Lower; everything must fit one main spec | Higher; philosophy / repository / runtime / UX / distribution can be reasoned about separately | Praxis |
| Immediate implementability | Very high for a bounded service | Medium; reader must merge multiple docs and resolve drift | Symphony |
| Drift resistance | Higher because there is only one main contract file | Lower because README, examples, UI, plan docs, and side specs can diverge | Symphony |
| Large-product modeling | More awkward when the product has many orthogonal boundaries | Better suited to large multi-boundary product modeling | Praxis |
| Context packing for AI tasks | Excellent when the task spans the whole service | Good when task-specific companion docs are supplied carefully | Tie; depends on discipline |
| Conflict resolution | Implicitly simple because there are fewer documents | Requires explicit precedence, and Praxis currently does not enforce it strongly enough | Symphony |
| Evolution friendliness | Single file can become dense and hard to maintain | Modular updates are easier, if canonicality is enforced | Praxis |
| Reviewability by specialty | Harder to review one aspect in isolation | Easier to review runtime, repository, or UX contracts separately | Praxis |
| Risk of silent contradiction | Lower | Higher | Symphony |

## 4. Strengths of the Praxis method

### 4.1 It scales better for a product than for a single daemon

Praxis is not one loop. It spans:
- repository contracts
- runtime-target contracts
- creation contracts
- library semantics
- benchmark/promotion semantics
- desktop/CLI surfaces
- release/distribution boundaries

A split package is structurally reasonable for that breadth.

### 4.2 It can package narrower task context better

When the task is only:
- runtime-slot logic
- source manifest rules
- creation draft rules
- release packaging

Praxis can pass only the relevant companion spec instead of the entire product universe.

### 4.3 It preserves philosophy and blueprint separately

That is useful when a product needs:
- a stable thesis
- a stable shape
- a canonical implementation contract
- detailed boundary appendices

Symphony’s single-file method is less natural for that.

## 5. Weaknesses of the current Praxis method

### 5.1 Canonicality exists on paper more than in practice

Praxis says:
- split package is intentional
- `03-SPEC.md` is canonical

But the repo still contains live contradictions:
- README says Gemini is first-class
- canonical runtime-target spec says Gemini is integration-only
- handoff says Decks are not top-level
- UX/IA says Decks are top-level
- blueprint says Agent Files are top-level
- plan refinement says Agent Files should be contextual
- desktop UI still exposes stale plan/guides top-level tabs

This means the split method currently suffers from **distributed contradiction**.

### 5.2 The companion docs sometimes create, not refine, cross-cutting policy

A companion spec should refine a boundary already anchored in the canonical spec.
In current Praxis, some companion docs still redefine cross-cutting product shape.
That is why `00`, `02`, `07`, and `plans/PRAXIS_FINAL_SPEC.md` can all disagree about top-level surfaces.

### 5.3 Non-canonical docs are too close to canonical gravity

The repo has:
- `specs/`
- `docs/`
- `plans/`
- `.agents/skills/` spec-generation skills

That is too many places for product truth unless each one has hard rules.
Right now the rules are not enforced tightly enough.

### 5.4 The split method needs tooling, and the tooling itself drifts

The repo already tries to solve this with:
- `write-spec`
- `spec-writing-standard`
- validators
- committed spec-writing standard docs

But the newer skill points to a missing path.
So the process layer has drift too.

### 5.5 Absolute local paths reduce portability

Several Praxis specs embed `/Users/axient/...` absolute paths.
That is the opposite of Symphony’s portable handoff style.

## 6. Strengths of the Symphony method

### 6.1 It is immediately operational

A coding agent can often implement Symphony directly from one file because the spec contains:
- file contract
- config schema
- state machine
- validation
- reconciliation
- error surface
- dynamic reload
- observability

### 6.2 It minimizes precedence confusion

There is little doubt about:
- what is canonical
- what the exact boundary is
- where to change the contract

### 6.3 It is highly agent-friendly

The spec is dense, typed, and operational.
It is close to executable reasoning.

## 7. Weaknesses of the Symphony method for a product like Praxis

### 7.1 It would get too dense if copied naively

If Praxis forced everything into one monolithic file, the result would likely become:
- harder to navigate
- harder to update safely
- harder to review by specialty
- harder to keep coherent across broad product boundaries

### 7.2 It is service-shaped, not product-shaped

Symphony is a scheduler/runner.
Praxis is a product spanning artifact discovery, runtime mapping, creation, library state, evaluation, and UI.
A pure single-file mirror is not the best authoring format for Praxis.

## 8. The right answer for Praxis: compiled monolith, modular sources

## 8.1 Decision

Praxis should **keep modular source specs**, but add a **compiled canonical monolith** for implementation handoff.

### Proposed structure

- `specs/03-SPEC.md` or `SPEC.md` becomes the single agent-facing canonical implementation contract.
- `00/01/02/04/05/06/07/08` remain modular source documents.
- each modular document may only:
  - refine one boundary
  - add detail
  - never override cross-cutting product truth without an explicit change record

### Why this is better

It keeps:
- Praxis’s decomposition benefits

while gaining:
- Symphony-like canonicality
- better agent handoff
- easier review of actual contract drift

## 9. Concrete rules Praxis should adopt

### Rule 1 — one compiled canonical implementation spec
Generate and maintain one monolithic implementation-facing spec from the split package.
That file is what Codex/Claude receive by default.

### Rule 2 — strict precedence ladder
Enforce:
1. current working core behavior
2. compiled canonical spec
3. modular spec package
4. committed plan docs
5. README/examples/UI copy

### Rule 3 — companion docs may refine, not redefine
A boundary-specific doc cannot invent:
- new top-level surfaces
- new runtime-target policy
- new storage authority
unless the canonical spec is updated first.

### Rule 4 — drift lint must become part of the spec method
Add repository checks for:
- banned stale terms (`guides`, `default_agents`, etc.)
- absolute local paths in canonical docs
- spec-tool broken internal links
- frontend/backend schema mismatch
- README/runtime-boundary mismatch

### Rule 5 — examples must be schema-conformant
Example manifests are not disposable.
They teach the contract.
If examples are stale, the spec method has failed.

### Rule 6 — every spec change needs an impact matrix
Each canonical change should say what must also be updated:
- README
- examples
- core model
- CLI flags
- desktop types
- validators
- spec-writing skills

## 10. Practical pros/cons summary

## Praxis split method — pros
- better modularity
- better specialization by boundary
- better fit for multi-surface product
- better targeted context packing

## Praxis split method — cons
- high drift risk
- lower immediate implementability
- easier to create conflicting truths
- requires stronger tooling/discipline than currently present

## Symphony monolith method — pros
- maximum clarity
- maximum implementation immediacy
- lower precedence confusion
- stronger agent handoff quality

## Symphony monolith method — cons
- weaker decomposition for large products
- can become dense
- less comfortable for broad product surface ownership

## 11. Bottom line

Symphony’s method is better at **canonicality and immediate implementation**.

Praxis’s method is better at **boundary decomposition for a larger product**.

The current problem in Praxis is not that the split method is inherently wrong.
The problem is that the repo does **not yet enforce enough rules** for the split method to behave like one spec.

The best upgrade is therefore:

> keep the split source package, but add a Symphony-style compiled canonical implementation spec and enforce drift checks around it.
