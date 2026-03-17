# PRAXIS Final Form Refinement

## 0. Purpose

This document does **not** claim current implementation status.
It records a **fact-grounded design decision** for the next stable product shape.

## 1. Fact base

### Current code reality
- current Rust core already implements deterministic inspect/plan/install/remove/sync/doctor behavior
- current Rust core already owns manifest + lock reconciliation
- current Rust core already composes managed blocks into runtime instruction files
- current desktop shell is behind the core contract
- current example manifest and terminology are partially stale

### Current strongest canonical direction
- `specs/03-SPEC.md` defines a local-first workspace with Library Plane + Runtime Plane + Evaluation Plane
- `specs/04-RUNTIME-TARGET-PROFILES.md` restricts first-class runtime-file targets to Codex and Claude
- `specs/05/06` define repository and creation boundaries that are larger than the current installer-only core

## 2. Final product definition

Praxis final form is fixed as:

> a **local-first skill workspace** that ingests external or internal artifact sources into a governed local library, projects selected artifacts into deterministic runtime outputs, and stores evaluation evidence used to promote or reject candidate artifacts.

## 3. Product planes

### 3.1 Runtime Plane
Status: partially implemented now.

Responsibilities:
- source inspection
- target/path resolution
- selection resolution
- plan preview
- copy-only install/remove/sync
- manifest desired state
- lock applied ownership
- agent-file slot composition
- drift/collision health checks

### 3.2 Library Plane
Status: not implemented in inspected code.

Responsibilities:
- local normalized artifact versions
- provenance
- source refresh snapshots
- internal drafts
- manual collections
- metadata queries
- installation-ready resolved versions

Authoritative storage:
- SQLite for metadata
- filesystem artifact store for artifact contents

### 3.3 Evaluation Plane
Status: not implemented in inspected code.

Responsibilities:
- benchmark suites
- candidate/current comparisons
- evidence persistence
- promotion recommendation inputs

## 4. Runtime boundary

### 4.1 First-class runtime-file targets
- Codex
- Claude

### 4.2 Integration-only target until formal promotion
- Gemini

Promotion condition:
Gemini becomes first-class only after all are true:
1. slot/root contract is documented canonically
2. planner/runtime mapping implements it
3. workspace path creation and apply semantics implement it
4. examples and UI reflect it
5. validation/doctor surface reports it consistently

## 5. Artifact boundary

Portable baseline:
- `SKILL.md`

Optional structured enrichment:
- `skill.json`
- `agents/openai.yaml`
- `skills.deck.json`
- future source metadata/template manifests only when explicitly recognized

Recipe boundary:
- recipe support remains explicit and narrow
- generic discovery remains the baseline

## 6. Workspace state authority

### Desired state
- manifest TOML

### Applied ownership
- lock JSON

### Library metadata
- SQLite

### Artifact contents
- filesystem artifact store

### Drafts / local collections
- internal source ids such as `internal:drafts` and `internal:collections`

## 7. UX boundary

### Primary surfaces
- Discover
- Library
- Create
- Benchmarks

### Utility surfaces
- Connections
- Health
- Settings

### Contextual/detail surfaces
- Plan preview
- Deck views
- Agent Files view/editor

Rationale:
- Plan should be a flow state, not a top-level destination
- Decks should be a view inside discovery/library, not a peer product center
- Agent Files are important but operational/contextual rather than a separate product identity

## 8. Authoritative implementation order

1. **Contract convergence**
   - remove stale guide-era contract leakage
   - align README/specs/examples/UI terminology
   - remove dead modules and broken spec-tool references

2. **Runtime Plane hardening**
   - finish path invariants
   - add contract/conformance tests
   - align desktop shell with backend

3. **Library Plane**
   - SQLite schema
   - artifact store
   - internal sources
   - provenance model

4. **Evaluation Plane**
   - benchmark schema
   - evidence persistence
   - promotion inputs

5. **Create system**
   - presets
   - drafts
   - preview tree
   - promotion writer
   - lineage

## 9. Never/always rules

### Always
- keep copy-only apply semantics
- preserve user-authored agent-file content
- preserve ownership-backed prune semantics
- keep runtime-native differences explicit
- separate fact from decision in product docs

### Never
- never silently overwrite unmanaged outputs
- never let README/examples/UI invent independent product contracts
- never call a runtime first-class before slot/root/apply/validation contracts exist
- never leave terminology migration half-finished

## 10. Acceptance criteria for “final form is fixed”

The final form is considered fixed only when all are true:
1. one canonical runtime-target boundary is stated consistently
2. one canonical surface model is stated consistently
3. README/examples/UI/schema match the canonical contract
4. runtime core passes conformance tests for plan/apply/remove/sync/agent-file composition
5. Library Plane storage authorities are implemented
6. Evaluation Plane evidence model is implemented
7. creation flows preserve repository contracts rather than inventing ad hoc outputs
