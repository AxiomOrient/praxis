# Praxis UX and Information Architecture Specification

Status: Draft v2

Purpose: Define the final navigation, primary surfaces, utility surfaces, and consequence-first interaction model for Praxis.

## 1. Problem Statement

Praxis cannot rely on a vague marketplace-style UI because the product's core value is consequence visibility: what exists, what is installed, what will change, and what owns what.

Important boundary:

- the UI is a consequence browser and management shell
- it is not a social marketplace, publishing portal, or detached design layer
- desktop and CLI must express the same operational contracts

## 2. Goals and Non-Goals

### 2.1 Goals

- expose source discovery and inspection
- expose Library as the top-level management surface for installed, imported, and draft artifacts
- expose Create as a first-class surface
- expose benchmark, health, and connection evidence
- keep install, remove, and compose flows legible before mutation

### 2.2 Non-Goals

- top-level navigation for transient plan state
- hiding source provenance or ownership details
- renaming surfaces in a way that obscures their contract

## 3. System Overview

### 3.1 Main Components

1. `Navigation Shell`
   - primary and utility navigation

2. `Discover`
   - source inspection and selection entry

3. `Library`
   - installed, imported, draft, augmented, outdated, and workspace-output views

4. `Create`
   - artifact creation and import entry

5. `Benchmark Lab`
   - candidate vs current comparison and promotion evidence

6. `Connections`
   - runtime detection and source access status

7. `Health`
   - drift, collisions, stale ownership, and invalid contract visibility

8. `Settings`
   - defaults, workspace settings, and advanced compatibility toggles

### 3.2 Abstraction Levels

1. `Navigation Layer`
   - top-level destinations

2. `Surface Layer`
   - Discover, Library, Create, Benchmark Lab

3. `Utility Layer`
   - Health, Connections, Settings

4. `Workspace Context Layer`
   - user workspace or repo workspace selection

5. `Flow Layer`
   - install, remove, create, deck review, agent-file composition, benchmark, promotion

### 3.3 External Dependencies

- CLI/core contracts exposed by the product
- runtime target rules from [04-RUNTIME-TARGET-PROFILES.md](04-RUNTIME-TARGET-PROFILES.md)
- library and planner state from [03-SPEC.md](03-SPEC.md)

### 3.4 Project Structure and Key Paths

- [03-SPEC.md](03-SPEC.md) - canonical product contract
- [07-UX-IA.md](07-UX-IA.md) - UI and IA contract
- [apps/praxis-desktop/src/App.svelte](../apps/praxis-desktop/src/App.svelte) - current desktop entry
- [apps/praxis-desktop/src/lib/components/](../apps/praxis-desktop/src/lib/components) - current desktop component surface

## 4. Core Domain Model

### 4.1 Entities

#### 4.1.1 SurfaceId

Definition:
Stable identifier for one top-level UI destination.

Fields:

- `id`
- `display_name`
- `kind` (`primary | utility`)

Lifecycle:
Declared by navigation shell.

#### 4.1.2 CardType

Definition:
Stable UI object type used by list and grid surfaces.

Fields:

- `id`
- `artifact_kind`
- `primary_actions[]`

Lifecycle:
Rendered in Discover and Library contextual views.

#### 4.1.3 FlowState

Definition:
User-visible stage within a multi-step flow.

Fields:

- `flow_id`
- `state_id`
- `next_actions[]`

Lifecycle:
Transitions through inspect, plan, apply, or create workflows.

#### 4.1.4 FilterState

Definition:
Named view over Library or other surfaces.

Fields:

- `id`
- `criteria`
- `badge_rules[]`

Lifecycle:
Selected by the user and re-evaluated as data changes.

### 4.2 Stable Identifiers and Normalization Rules

- primary surface names are stable: Discover, Library, Create, Benchmark Lab
- utility surface names are stable: Health, Connections, Settings
- `Installed`, `Imported`, and `Draft` are mutually exclusive primary presence filters in Library
- additional state such as `outdated`, `invalid`, or `benchmarked` appears as additive badges, not replacement presence states

## 5. Domain Contract

### 5.1 Navigation and Surface Contract

Primary surfaces are:

- Discover
- Library
- Create
- Benchmark Lab

Utility surfaces are:

- Health
- Connections
- Settings

Agent Files are not a top-level surface.
Agent-file outputs appear in workspace settings, plan or apply review, and Library/workspace-detail surfaces.

The desktop shell must expose a global workspace context selector:

- User Workspace
- Repo Workspace

Rejected top-level names include:

- Catalog
- Guides
- Agent Files
- Benchmarks
- Doctor
- Plan

## 6. Configuration and Input Contract

### 6.1 Precedence and Defaults

UI defaults may come from:

1. current explicit user selection
2. last active view
3. settings defaults

### 6.2 Validation and Coercion

The UI must validate:

- selected filters map to real Library states
- plan previews exist before apply is confirmed
- workspace context is explicit before workspace-scoped mutation
- agent-file outputs are previewable from workspace settings or review surfaces

## 7. Lifecycle or State Model

### 7.1 States

1. `inspect`
   - source is being reviewed

2. `preview`
   - plan or effective-file consequences are visible

3. `confirm`
   - user is about to mutate state

4. `applied`
   - change completed and the affected workspace surfaces should refresh

### 7.2 Transitions and Guards

- Discover inspect -> preview plan -> confirm -> apply
- Library selection -> preview removal or update consequences -> confirm -> apply
- Library deck subview -> preview membership and install consequences -> confirm -> apply
- Create editing -> preview -> confirm -> save or promote
- Benchmark Lab review -> compare candidates -> confirm promotion or hold

## 8. Primary Workflows and Reconciliation

### 8.1 Primary Flow

Discover flow:

1. enter source
2. inspect source
3. choose artifacts
4. preview plan
5. confirm apply
6. land in Library with contextual deck or workspace detail depending on the selected artifact

### 8.2 Failure or Retry Branches

- plan conflicts keep the user in preview until resolved
- invalid sources remain visible in Discover and Health
- failed apply returns to a visible previewable state instead of silent reset

## 9. Storage, Ownership, and Safety Boundaries

### 9.1 Managed Locations

The UI must expose:

- source provenance
- target paths
- runtime file destinations
- managed vs unmanaged status

### 9.2 Destructive Boundaries

- UI must never hide which files will change
- destructive actions require preview before confirmation
- user-authored runtime content must remain visually distinct from managed blocks

## 10. Execution or Interface Contract

### 10.1 Invocation Contract

Desktop is a shell over the same operations expressed by CLI.

- Discover maps to inspect/plan/apply
- Library maps to remove/sync/update and artifact/runtime state browsing
- Create maps to create/import/promote
- contextual deck views map to collection review and deck-driven install
- Benchmark Lab maps to benchmark execution and promotion review

## 11. External Integration Contract

### 11.1 Required Operations

- source inspection
- plan preview
- apply/remove/sync operations
- agent-file output preview
- benchmark and promotion review

## 12. Context Packaging and Prompt Inputs

### 12.1 Required Context

Always provide:

- [03-SPEC.md](03-SPEC.md)
- [07-UX-IA.md](07-UX-IA.md)

### 12.2 Task-Specific Context

Only include:

- the specific surface being changed
- the corresponding CLI/core contract
- relevant component files if UI implementation is involved

### 12.3 Context Reduction Rules

- one surface or one flow per prompt when possible
- do not include unrelated benchmark or distribution rules when fixing a Discover-only issue

## 13. Logging, Status, and Observability

### 13.1 Operator-Visible Signals

- source provenance
- install state
- target file consequences
- invalid and outdated badges
- benchmark evidence
- current workspace context

### 13.2 Logs and Traces

- UI actions that mutate state should emit progress and failure surfaces that map back to CLI/core operations
- Health should expose collisions, drift, invalid contracts, and stale ownership explicitly

## 14. Failure Model and Recovery Strategy

### 14.1 Failure Classes

- `source_invalid`
- `plan_conflict`
- `runtime_slot_invalid`
- `apply_failed`
- `benchmark_failed`

### 14.2 Recovery Behavior

- invalid sources remain inspectable
- plan conflicts remain previewable and resolvable
- failed apply or composition returns the user to a visible corrective state

## 15. Safety, Boundaries, and Human Approval Policy

### 15.1 Always Do

- show exact consequences before mutation
- preserve source provenance visibility
- preserve managed vs user-owned distinction
- keep agent-file outputs contextual instead of promoting them into top-level navigation

### 15.2 Ask First

- any action that removes managed artifacts
- any apply that affects agent files with user content
- any promotion action from Benchmark Lab into production-ready state

### 15.3 Never Do

- never hide source origin
- never make Plan a top-level managed destination
- never make Agent Files a top-level surface
- never mutate runtime files without a visible preview step

## 16. Reference Algorithms and Task Decomposition

### 16.1 Reference Algorithm

1. user enters a surface
2. surface loads current state
3. user selects a bounded action
4. UI renders consequence preview
5. user confirms mutation
6. UI refreshes the affected surfaces

### 16.2 Task Units and Parallelism

- surface-specific work should be separated by Discover, Library, Create, Benchmark Lab, and utilities
- UI work may proceed in parallel across unrelated surfaces, but consequence preview and mutation confirmation must remain sequential in one flow

## 17. Validation, Commands, and Success Criteria

### 17.1 Commands

- `praxis inspect <source>` - Discover backing operation
- `praxis plan ...` - preview backing operation
- `praxis apply ...` - mutation backing operation
- `praxis doctor` - Health backing operation
- `praxis benchmark run <suite>` - Benchmark Lab backing operation

### 17.2 Validation Matrix

- every primary surface has a bounded job
- plan remains transient rather than a top-level destination
- Library filters preserve presence-state semantics
- workspace context scopes installs, agent-file outputs, and doctor results
- agent files remain contextual rather than top-level

### 17.3 Acceptance and Conformance Gates

- desktop and CLI describe the same consequences
- every destructive path includes preview before confirmation
- source provenance and target-file consequences are visible in the UI

## 18. Implementation Checklist and Change Control

### 18.1 Required for Conformance

- primary and utility navigation implemented
- Discover, Library, Create, and Benchmark Lab surfaces implemented
- consequence preview implemented before mutation
- Health and Connections implemented as utility surfaces

### 18.2 Recommended Extensions

- richer empty-state guidance
- more advanced comparison views for benchmarks and drafts
- additional accessibility and keyboard-flow refinement

### 18.3 Spec Update Triggers

- top-level surface list changes
- Library or contextual deck state model changes
- workspace context model changes
- plan or apply semantics change in CLI/core
