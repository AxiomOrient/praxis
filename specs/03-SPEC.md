# Praxis Product Specification

Status: Draft v2 (canonical)

Purpose: Define Praxis as an AI-facing implementation contract for a local-first skill workspace built from Library Plane, Runtime Plane, and Evaluation Plane while preserving the current deterministic installer core.

## 1. Problem Statement

Praxis exists because the current source-manager model solves deterministic install, but does not yet solve local library, authoring lifecycle, provenance, and evaluation as one coherent system.

The product solves five connected problems:

- external GitHub and local repositories are useful origins, but they are not a sufficient local library model
- installation and removal are unsafe when ownership is ambiguous
- persistent runtime instruction files are scattered across runtimes and treated as second-class outputs
- fork, rename, augment, and promotion are fragmented across ad hoc tools
- benchmark evidence and promotion decisions are not persisted with the artifacts they evaluate

Important boundary:

- Praxis is a local-first skill workspace, not a hosted registry or community artifact home.
- External repositories remain authoritative origins for portable source artifacts, but local library versions are authoritative for workspace planning.
- Praxis owns source intake, library storage, runtime reconciliation, creation, evaluation, and promotion workflows around those artifacts.

## 2. Goals and Non-Goals

### 2.1 Goals

- inspect external GitHub, local, and internal sources
- discover portable skills, decks, guide outputs, and agent-file templates from those sources
- preview exact filesystem and runtime-file consequences before mutation
- apply copy-only installs with deterministic ownership and residue-free removal
- manage persistent instruction files for supported runtimes as first-class outputs
- support draft creation, import, fork, rename, augment, and promotion of artifacts
- preserve runtime-specific differences across Codex, Claude Code, and Gemini integrations without pretending one universal runtime model exists
- provide contract-consistent CLI and desktop surfaces
- make validation, drift, collisions, benchmark evidence, and provenance visible
- persist library metadata, jobs, and benchmark history across restarts without requiring a resident daemon

### 2.2 Non-Goals

- general-purpose package registry
- hosted multi-tenant control plane
- undocumented runtime filename aliases or undeclared Gemini runtime file targets as first-class product contracts
- complete editors for every vendor-native configuration file
- replacing native agent products
- silently flattening runtime differences into one fake universal model

## 3. System Overview

### 3.1 Main Components

1. `Library Plane`
   - resolves GitHub, local, and internal sources into normalized local versions
   - stores metadata in SQLite and artifact contents in filesystem state

2. `Runtime Plane`
   - preserves the current deterministic planner, reconciler, guide merge, manifest, and lock model
   - maps library-resolved versions into concrete runtime outputs

3. `Evaluation Plane`
   - runs benchmark suites, persists results, and computes promotion recommendations

4. `Creator Layer`
   - creates, forks, renames, augments, imports, and promotes skills and decks

5. `Job Engine`
   - persists queued work and progresses it through cooperative workers without requiring a resident daemon

6. `Connections and Health`
   - checks toolchain availability, auth posture, path writability, drift, and conflicts

7. `CLI Surface`
   - provides authoritative operational commands

8. `Desktop Surface`
   - provides a visual shell over the same contracts

### 3.2 Abstraction Levels

1. `Source Layer`
   - GitHub, local, and internal origins

2. `Snapshot Layer`
   - exact fetched or captured source state

3. `Library Version Layer`
   - immutable normalized skill, deck, guide, and bundle versions

4. `Workspace Selection Layer`
   - desired state for one workspace

5. `Runtime Reconciliation Layer`
   - plan, apply, prune, lock, and guide merge

6. `Evaluation Layer`
   - benchmark results and promotion evidence

7. `Observation Layer`
   - Library, My Skills, Decks, Health, Connections, and Benchmark Lab

### 3.3 External Dependencies

- GitHub or local filesystem access for source resolution
- local filesystem access for SQLite state, library artifacts, manifests, locks, drafts, and runtime target paths
- documented runtime contracts for Codex and Claude Code runtime outputs
- Gemini detection and creator or evaluation integrations where runtime-file contracts are not yet first-class
- optional benchmark runners, judge models, or human-review workflows

### 3.4 Project Structure and Key Paths

- [03-SPEC.md](/Users/axient/repository/praxis/specs/03-SPEC.md) — canonical product contract
- [04-RUNTIME-TARGET-PROFILES.md](/Users/axient/repository/praxis/specs/04-RUNTIME-TARGET-PROFILES.md) — runtime mapping rules
- [05-REPOSITORY-CONTRACTS.md](/Users/axient/repository/praxis/specs/05-REPOSITORY-CONTRACTS.md) — source and repository contract details
- [06-CREATION-SYSTEM.md](/Users/axient/repository/praxis/specs/06-CREATION-SYSTEM.md) — creation and draft workflow contracts
- [07-UX-IA.md](/Users/axient/repository/praxis/specs/07-UX-IA.md) — surface model and IA
- [08-DISTRIBUTION.md](/Users/axient/repository/praxis/specs/08-DISTRIBUTION.md) — release boundary
- [apps/praxis-desktop/](/Users/axient/repository/praxis/apps/praxis-desktop) — current desktop implementation surface

## 4. Core Domain Model

### 4.1 Entities

#### 4.1.1 SourceRef

Definition:
One external or internal artifact source.

Fields:

- `kind` (`github | local | internal`)
- `owner?`
- `repo?`
- `ref?`
- `subdir?`
- `path?`
- `canonical_id`

Lifecycle:
Created during inspect or source selection. Remains stable across equivalent input spellings including `internal:drafts` and `internal:collections`.

#### 4.1.2 Scope

Definition:
One managed installation boundary.

Fields:

- `id` (`repo | user`)
- `root_path`

Lifecycle:
Selected per operation and used for manifest, lock, and runtime mapping.

#### 4.1.3 Catalog

Definition:
Normalized scan output for one source.

Fields:

- `source`
- `skills[]`
- `decks[]`
- `agent_file_templates[]`
- `warnings[]`
- `notes[]`
- `recipe?`

Lifecycle:
Produced by inspect and refresh operations. Recomputed on sync or update.

#### 4.1.4 Skill

Definition:
Minimum installable capability unit.

Fields:

- `id`
- `name`
- `description`
- `path`
- `display_name?`
- `category?`
- `tags[]`
- `sidecars`

Lifecycle:
Discovered from source, selected into a plan, then available, installed, or drafted in Library state.

#### 4.1.5 Deck

Definition:
Named grouping of skills.

Fields:

- `id`
- `title`
- `description?`
- `members[]`
- `origin` (`declared | synthesized | recipe | draft`)

Lifecycle:
Discovered, synthesized, or drafted; may influence selection resolution without being a copied runtime artifact itself.

#### 4.1.6 AgentFileTemplate

Definition:
Reusable managed instruction block or seed for one or more runtime slots.

Fields:

- `id`
- `title`
- `description`
- `path`
- `slots[]`
- `priority`
- `origin` (`declared | recipe | draft`)

Lifecycle:
Discovered or drafted, then selected into slot composition.

#### 4.1.7 TargetProfile

Definition:
Explicit runtime mapping policy.

Fields:

- `id`
- `agents[]`
- `skill_roots[]`
- `agent_file_slots[]`
- `shared_roots?`

Lifecycle:
Chosen by the user or defaults; used by planner and reconciler. Runtime-file install targets are currently first-class for Codex and Claude; Gemini remains an integration target until a concrete runtime-file contract is adopted.

#### 4.1.8 Selection

Definition:
Desired state for one canonical source within one scope.

Fields:

- `source_id`
- `scope`
- `target_profile`
- `all`
- `decks[]`
- `skills[]`
- `exclude_skills[]`
- `agent_file_templates[]`

Lifecycle:
Built during inspect/apply flows and persisted into manifest desired state.

#### 4.1.9 Plan

Definition:
Pure preview object derived from desired state plus current state.

Fields:

- `selection`
- `resolved_skills[]`
- `resolved_agent_file_templates[]`
- `copy_actions[]`
- `agent_file_actions[]`
- `warnings[]`
- `conflicts[]`
- `notes[]`
- `summary`

Lifecycle:
Created by planner; must not mutate state.

#### 4.1.10 Manifest

Definition:
Desired-state record stored by Praxis.

Fields:

- `scope`
- `source_records[]`

Lifecycle:
Written on successful apply and updated on remove, sync, and selection changes.

#### 4.1.11 Lock

Definition:
Applied-state ownership record.

Fields:

- `managed_paths[]`
- `managed_blocks[]`
- `hashes[]`
- `ownership[]`

Lifecycle:
Written only after successful reconciliation and used for repair, prune, and remove.

#### 4.1.12 LibraryEntry

Definition:
Normalized record shown in Library-backed surfaces such as My Skills and Decks.

Fields:

- `artifact_kind` (`skill | deck | agent_file_template | draft`)
- `artifact_id`
- `source_id?`
- `origin` (`source | imported | draft | recipe`)
- `presence_state` (`available | installed | draft`)
- `status_flags[]`
- `target_profiles[]`
- `updated_at?`

Lifecycle:
Derived from source inspection, manifest, lock, and drafts.

#### 4.1.13 BenchmarkSuite

Definition:
Named evaluation suite for one artifact class.

Fields:

- `id`
- `artifact_kind`
- `tasks[]`
- `checks[]`
- `baseline?`
- `candidate?`

Lifecycle:
Configured or discovered, then executed to produce promotion evidence.

### 4.2 Stable Identifiers and Normalization Rules

- source resolution must produce one canonical source id before any manifest, cache, or lock lookup
- internal source ids `internal:drafts` and `internal:collections` are reserved and stable
- skill `name` is the portable identity across compatible repositories
- deck ids are unique within one catalog
- agent-file template ids are unique within one source
- runtime slot ids are globally stable
- Library presence state is singular per scope view, while status flags are additive

## 5. Domain Contract

### 5.1 Source and Repository Contract

- Praxis must support portable open-standard sources containing one or more `SKILL.md` directories
- optional manifests such as `skills.deck.json`, `praxis.source.json`, and `agent-files/manifest.json` enrich UX but are not required for basic usefulness
- library metadata is authoritative in SQLite, while artifact contents remain authoritative in the filesystem artifact store
- workspace desired state is authoritative in manifest TOML and applied ownership is authoritative in lock JSON
- recipe-backed sources are allowed when generic discovery alone is insufficient
- arbitrary Markdown files must not be inferred as templates unless a manifest or recipe says so

## 6. Configuration and Input Contract

### 6.1 Precedence and Defaults

Decision order is:

1. explicit user selection
2. current scope choice
3. chosen target profile
4. workspace manifest desired state
5. library-resolved latest or pinned version
6. inferred defaults

Explicit user choice must win over inferred defaults.

### 6.2 Validation and Coercion

At minimum, Praxis must validate:

- `SKILL.md` exists for every discovered skill
- required frontmatter `name` and `description` exist
- normalized skill identity matches directory identity when the source claims structured metadata
- declared deck members resolve to discovered skill names
- declared template slots resolve to known slot ids
- selected target profile is compatible with requested runtimes and roots

## 7. Lifecycle or State Model

### 7.1 States

1. `available`
   - artifact is known from a source but not currently installed in the selected scope

2. `installed`
   - artifact is implied by manifest and lock in the selected scope

3. `draft`
   - artifact exists locally in Praxis draft state and is not yet promoted into a source-owned contract

4. `benchmarked`
   - artifact has persisted evaluation evidence and promotion signals

### 7.2 Transitions and Guards

- inspect source -> create `available`
- apply selection -> move selected entries to `installed`
- remove or prune -> return entries to `available` when the source is still known
- create or import -> create `draft`
- promote draft -> convert to `available` or `installed` depending on immediate selection
- invalid entries remain inspectable; they do not disappear from Library
- re-inspecting the same canonical source must not duplicate identity

## 8. Primary Workflows and Reconciliation

### 8.1 Primary Flow

1. resolve source input to one canonical source id
2. capture or refresh source snapshot
3. normalize library versions and metadata into SQLite plus artifact storage
4. compute desired selection for one scope and target profile
5. compute deterministic copy, prune, and slot-composition actions
6. stop if blocking conflicts exist
7. write outputs
8. write manifest and lock to the same successful reconciliation result
9. update Library-backed views, Health, guide output views, and Benchmark surfaces

### 8.2 Failure or Retry Branches

- unresolved sources or invalid contracts block apply but preserve desired state for later repair
- missing managed outputs may be repaired only when desired state still implies them
- remove and sync must prune outputs no longer implied by desired state
- `update` is an alias of `sync` in v1 and must not introduce a second update model

## 9. Storage, Ownership, and Safety Boundaries

### 9.1 Managed Locations

- SQLite DB — library metadata, provenance, jobs, benchmark history, and connection state
- library artifact store — immutable skill, deck, guide, and bundle contents
- manifest — desired workspace state
- lock — applied ownership state
- cache — fetched source material
- drafts — not-yet-promoted local artifacts
- runtime roots and slot files — mutation targets

Every managed output must be attributable to a `(scope, source_id, artifact_id)` ownership tuple.

### 9.2 Destructive Boundaries

- installs must be copy-only
- unmanaged paths must not be overwritten implicitly
- shared roots must not receive duplicate copies of the same resolved artifact set
- partial failure must not leave lock state claiming outputs that were not written

## 10. Execution or Interface Contract

### 10.1 Invocation Contract

Praxis exposes one authoritative operational contract through CLI and one visual shell through desktop.

- desktop must map to the same core operations as CLI
- inspect, plan, apply, remove, sync, create, fork, augment, promote, benchmark, doctor, and jobs work are first-class operations
- CLI naming is authoritative for automation and tests

## 11. External Integration Contract

### 11.1 Required Operations

- GitHub, local, or internal source resolution
- runtime mapping for Codex and Claude Code
- Gemini detection plus creator and evaluation integration where applicable
- optional benchmark execution against current and candidate artifacts

For each integration, Praxis must keep vendor-native semantics explicit rather than flattening them into one generic abstraction.

## 12. Context Packaging and Prompt Inputs

### 12.1 Required Context

Always provide:

- [03-SPEC.md](/Users/axient/repository/praxis/specs/03-SPEC.md) for the canonical contract
- the specific companion spec that matches the task domain:
  - runtime mapping -> [04-RUNTIME-TARGET-PROFILES.md](/Users/axient/repository/praxis/specs/04-RUNTIME-TARGET-PROFILES.md)
  - source contracts -> [05-REPOSITORY-CONTRACTS.md](/Users/axient/repository/praxis/specs/05-REPOSITORY-CONTRACTS.md)
  - creation/drafts -> [06-CREATION-SYSTEM.md](/Users/axient/repository/praxis/specs/06-CREATION-SYSTEM.md)
  - UX/surfaces -> [07-UX-IA.md](/Users/axient/repository/praxis/specs/07-UX-IA.md)
  - release boundary -> [08-DISTRIBUTION.md](/Users/axient/repository/praxis/specs/08-DISTRIBUTION.md)

### 12.2 Task-Specific Context

Only include:

- the relevant implementation files for the bounded task
- the relevant runtime or repository contract slice
- the current failure logs or test output if debugging

Do not dump unrelated companion specs into every prompt.

### 12.3 Context Reduction Rules

- use one bounded implementation target per prompt whenever possible
- prefer section-level excerpts over the full spec set for narrow tasks
- start a new planning or implementation prompt when the task domain changes materially

## 13. Logging, Status, and Observability

### 13.1 Operator-Visible Signals

- source warnings
- plan conflicts
- drift indicators
- managed ownership and provenance
- benchmark results and promotion evidence

### 13.2 Logs and Traces

- inspect, plan, apply, remove, sync, create, augment, benchmark, and promotion operations should emit operator-visible status
- health output must be reproducible from SQLite metadata, manifest, lock, runtime paths, and current source state
- job progress must be recoverable after restart without requiring a resident daemon

## 14. Failure Model and Recovery Strategy

### 14.1 Failure Classes

- `source_unreachable`
- `source_invalid`
- `skill_invalid`
- `deck_invalid`
- `template_invalid`
- `unknown_selection_member`
- `target_profile_invalid`
- `unmanaged_collision`
- `shared_root_conflict`
- `lock_drift`
- `recipe_error`
- `job_lease_expired`
- `benchmark_incomplete`

### 14.2 Recovery Behavior

- invalid source or invalid selection blocks apply
- plan should remain available wherever enough information exists to explain the failure
- unresolved sources preserve desired manifest state rather than being silently dropped
- drift may be repaired only when the source remains valid and desired state still implies the missing output
- invalid drafts remain editable but cannot be promoted
- expired job leases may be reclaimed by another worker after the persisted lease timeout

## 15. Safety, Boundaries, and Human Approval Policy

### 15.1 Always Do

- preserve deterministic ordering in planner and composer output
- preserve user-authored runtime instruction content when managed blocks change
- refuse implicit overwrite of unmanaged content
- keep runtime-specific behavior explicit in the spec and implementation
- preserve local library provenance when creating forked or augmented versions

### 15.2 Ask First

- introducing a new runtime target
- changing canonical slot ids or target profile defaults
- turning Gemini into a first-class runtime-file target
- changing manifest or lock format in a way that affects persisted state
- redefining removal behavior where user-owned content may be affected

### 15.3 Never Do

- never silently overwrite unmanaged paths
- never claim one runtime contract applies to all runtimes when official docs differ
- never drop desired manifest state because a source is temporarily unavailable
- never promote invalid drafts or invalid source contracts as if they were conformant
- never require a permanently running daemon for normal product operation

## 16. Reference Algorithms and Task Decomposition

### 16.1 Reference Algorithm

1. resolve source input
2. capture or refresh source snapshot
3. normalize library versions and metadata
4. compute desired selection
5. derive deterministic runtime actions
6. block on conflicts
7. write managed outputs
8. write manifest and lock
9. refresh observational surfaces and persisted job state

### 16.2 Task Units and Parallelism

- source resolution, library normalization, planner determinism, runtime mapping, evaluation, and UI surface work should be specifiable as separate bounded tasks
- independent runtime adapters may be implemented in parallel when they do not share write paths
- planner and reconciler logic must remain sequential at the point where a single reconciliation unit writes manifest, lock, and outputs

## 17. Validation, Commands, and Success Criteria

### 17.1 Commands

- `praxis inspect <source>` — resolve and catalog one source
- `praxis source sync [--source <id>]` — refresh source snapshots
- `praxis library list` — inspect normalized local library state
- `praxis plan --source <source> --scope <repo|user> --target-profile <id>` — compute exact consequences without mutation
- `praxis apply --source <source> --scope <repo|user> --target-profile <id>` — reconcile desired state
- `praxis remove --source <source> --scope <repo|user>` — remove managed state implied by the source selection
- `praxis sync --scope <repo|user>` — re-scan selected sources and reconcile drift
- `praxis doctor --scope <repo|user>` — report collisions, drift, stale ownership, and invalid contracts
- `praxis skill fork <skill-version-id>` — create a derived local version
- `praxis skill augment <skill-version-id>` — create an augmented local version
- `praxis benchmark run <suite>` — evaluate candidate artifacts against a suite
- `praxis jobs work` — progress persisted jobs cooperatively

### 17.2 Validation Matrix

- source resolution and catalog parsing must normalize equivalent source spellings and reject broken contracts
- library storage must preserve SQLite metadata integrity and filesystem artifact provenance
- planning and reconciliation must produce deterministic action ordering and block unmanaged collisions
- agent-file composition must preserve user content and deterministic managed-block ordering
- Library state must expose exactly one presence state plus additive status flags
- runtime target integration must reject incompatible slot or target-profile selections
- cooperative jobs must recover cleanly after restart without daemon-only assumptions

### 17.3 Acceptance and Conformance Gates

- the canonical spec must pass `python3 /Users/axient/repository/praxis/.agents/skills/spec-writing-standard/scripts/check_spec_standard.py /Users/axient/repository/praxis/specs/03-SPEC.md`
- companion docs must not contradict the contracts defined here
- CLI and desktop surfaces must describe the same operational consequences
- destructive operations must be blocked when ownership is ambiguous

## 18. Implementation Checklist and Change Control

### 18.1 Required for Conformance

- canonical source resolution implemented
- portable skill discovery plus optional manifest enrichment implemented
- SQLite-backed library metadata and filesystem artifact storage implemented
- deterministic planning and reconciliation implemented
- copy-only ownership-backed install and prune behavior implemented
- managed slot composition with preserved user content implemented
- Library lifecycle states and additive status flags implemented
- validation surfaces for source, selection, drift, and template failures implemented
- persisted cooperative jobs implemented without a resident-daemon requirement

### 18.2 Recommended Extensions

- richer benchmark evidence and promotion workflows
- draft diffing and augmentation provenance history
- broader runtime-target diagnostics and installation guidance

### 18.3 Spec Update Triggers

- runtime target contracts change
- manifest or lock schema changes
- source compatibility tier rules change
- library storage model or internal source ids change
- draft or benchmark systems become first-class canonical surfaces
- CLI and desktop semantics diverge and need reconciliation
