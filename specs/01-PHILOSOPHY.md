# 01. Philosophy

## 1. Product thesis

The problem is no longer “how to write more prompts for a coding agent.”
The problem is how to make a repository **legible, repeatable, and governable** for multiple agent runtimes.

Praxis exists to solve that problem.

## 2. Simplicity definition

Simplicity does **not** mean fewer features.
Simplicity means:

- one clear center of gravity
- one exact ownership model
- one obvious install flow
- one predictable set of runtime targets
- no hidden residue

A simple product may still be deep.
It becomes simple by being structurally exact.

## 3. First principles

### 3.0 Repository-owned truth

Agent-facing knowledge must live in versioned repositories or documented runtime files, not in transient chat history or scattered notes. That is the practical lesson behind harness engineering and the official instruction-file models used by modern coding agents.[R2][R4][R6][R8]

### 3.1 External repositories are the source of truth

Praxis is a management plane, not the canonical home of community skills.
Skills live in repositories; Praxis makes them legible and manageable.

### 3.2 Open standard first

The minimum shared capability unit is a directory containing `SKILL.md` with required metadata, matching the open agent skills model adopted by Codex and Gemini CLI and followed by Claude Code with extensions.[R3][R5][R7]

### 3.3 Agent files are first-class runtime artifacts

Official runtimes do not treat `AGENTS.md`, `CLAUDE.md`, or `GEMINI.md` as optional notes.
They are persistent instruction surfaces loaded into sessions and scoped by location.[R2][R6][R8]

Therefore Praxis must manage them explicitly.

### 3.4 Install semantics must be exact

The filesystem is the final truth surface.
If ownership is ambiguous, update and removal become unsafe.
Therefore Praxis is:

- copy-only
- lock-backed
- prune-capable
- conflict-aware

### 3.5 One source install per scope

The unit of desired state is not an individual copied folder.
It is the selection record for one canonical source id within one scope.

This prevents split ownership and duplicate records.

### 3.6 Inspect -> Choose -> Preview Plan -> Apply

Users must see consequences before mutation.
Plan is not ornamental UX; it is part of the product’s safety model.

### 3.7 Shared roots where the ecosystem already shares them

Codex and Gemini both support the open-standard `.agents/skills` root.[R3][R7]
Praxis should use that overlap to remove duplication where it is real, not where it is imagined.

### 3.8 Vendor-native features are preserved, not flattened away

Claude supports native skill location rules and richer skill behavior; Codex has AGENTS layering and override rules; Gemini has hierarchical `GEMINI.md`, `/memory`, and native `.gemini` settings.[R2][R5][R6][R7][R8]

Praxis must preserve those differences instead of pretending they are identical.

### 3.9 Creation belongs inside the product

If users can only install but cannot author or adapt, the product remains incomplete.
Create is not a nice-to-have; it is part of the management plane.

### 3.10 Benchmark before promotion

The right place for comparison is artifact-level evaluation, not gut feel.
Benchmarks exist to compare current vs candidate skills, decks, and agent-file templates before promotion.

## 4. Rejected alternatives

### 4.1 Registry-first architecture

Rejected because it duplicates metadata and displaces the actual source repository.

### 4.2 Desktop-only architecture

Rejected because it weakens automation and auditability.

### 4.3 Symlink-first installation

Rejected because it weakens ownership, prune, and portability guarantees.

### 4.4 “Guides” as a secondary concept

Rejected because it understates the role of persistent instruction files in actual agent runtimes.

### 4.5 Top-level Decks and top-level Plan

Rejected because they describe intermediate states, not enduring jobs.

## 5. Canonical quality bar

Praxis is complete when a user can do the following with no ambiguity:

1. inspect a source
2. understand what is installable
3. choose only what matters
4. preview exactly what will change
5. apply once
6. remove without residue
7. see and edit runtime instruction files safely
8. create new artifacts without leaving the product

That is the bar.
