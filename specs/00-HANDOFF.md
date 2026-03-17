# 00. Handoff

## Status

Praxis is no longer defined as a “skill pack” product.
Praxis is the **management plane** for external agent capabilities and persistent agent instruction files.

## One-sentence definition

Praxis inspects external repositories, lets users choose decks, atomic skills, and agent files, previews the exact result, then applies a deterministic copy-only reconcile into agent-specific runtime locations.

## What must not be re-opened

1. **Praxis is not a registry-first product.**
   External repositories remain the source of truth.
2. **Praxis is not a symlink manager.**
   Install semantics are copy-only.
3. **One source install per scope is mandatory.**
   A canonical source id appears at most once in a given scope.
4. **`Guides` is retired language.**
   The correct product noun is **Agent Files**.
5. **Decks are not top-level navigation.**
   They live inside Library and Discover flows.
6. **Plan is not top-level navigation.**
   `Inspect -> Choose -> Preview Plan -> Apply` is the canonical interaction flow.
7. **Open-standard skill compatibility is the default posture.**
   Vendor-native enrichments are additive, not foundational.
8. **Non-canonical filenames are not first-class.**
   Praxis manages documented default names first; aliases are opt-in advanced compatibility, not the product center.

## Final product center of gravity

Praxis has four equal pillars:

1. **Discover** external sources and inspect what they contain.
2. **Library** managed artifacts, installed outputs, and runtime workspace state.
3. **Create** new skills, decks, and agent-file templates with compatibility presets.
4. **Benchmarks** compare and promote candidate artifacts with explicit evidence.

Benchmarks exist to compare and promote artifacts, but they do not replace the management plane.

## Canonical navigation

Primary navigation:

- Discover
- Library
- Create
- Benchmarks

Utility navigation:

- Connections
- Health
- Settings

Transient flows:

- Inspect
- Preview Plan
- Apply
- Deck Views
- Agent Files Editor
- Conflict Review
- Update Review

## Canonical nouns

- **Source** — external GitHub repo or local directory
- **Catalog** — scanned representation of one source
- **Skill** — minimum installable capability unit
- **Deck** — named set of skills
- **Agent File Template** — reusable instruction file block or file seed
- **Target Profile** — runtime-specific install mapping
- **Selection** — chosen decks, skills, templates, and targets for a source
- **Plan** — pure preview object
- **Manifest** — desired state
- **Lock** — applied ownership and hashes
- **Library** — locally managed artifacts across sources

## Final product boundary

Praxis manages:

- external source intake
- skill/deck selection and installation
- deterministic update/remove/prune
- agent-file composition for supported runtimes
- creation of new portable artifacts
- compatibility-aware packaging
- health and benchmark surfaces

Praxis does not manage:

- a public marketplace backend
- cloud accounts or multi-tenant sync
- arbitrary vendor config editing in full generality
- plugin/extension ecosystems beyond the documented runtime targets
- undocumented alias files as if they were official runtime contracts

## The key product correction

The decisive shift is this:

> Praxis is not only a skill installer. It is the system that keeps the agent workspace legible.

That means skills and agent files must be managed together, because official runtimes treat `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` as persistent instruction surfaces loaded into agent context.[R2][R6][R8]

## Final implementation posture

- Rust core is authoritative.
- CLI is the operational truth surface.
- Desktop is a visual shell over the same contracts.
- The product remains macOS-first.
- The product remains repository-friendly and offline-capable after installation.

## Reading rule for the next agent

Do not start from the UI.
Read `01-PHILOSOPHY.md`, then `02-BLUEPRINT.md`, then `03-SPEC.md`.
Everything else is a subordinate expansion of those documents.
