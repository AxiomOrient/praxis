# Praxis Creation System Specification

Status: Draft v2

Purpose: Define the first-class creation, import, draft, fork, rename, augment, preview, and promotion system for skills, decks, source metadata, and agent-file templates.

## 1. Problem Statement

Praxis cannot be only an installer.
Users need to author, adapt, and promote artifacts with the same determinism used for inspection and apply.

Important boundary:

- Create is an artifact-authoring and draft-management system.
- It is not a full generic IDE or rich freeform documentation suite.
- Create must emit valid repository contracts, not project-specific one-off blobs.

## 2. Goals and Non-Goals

### 2.1 Goals

- create new portable skills
- fork, rename, and augment imported or drafted skills
- create declared decks
- create reusable agent-file templates
- create source metadata drafts
- import existing skill folders
- preview exact output trees before write or promotion
- preserve draft validity and deterministic reload

### 2.2 Non-Goals

- emitting invalid artifacts for the selected preset
- hiding compatibility consequences behind “smart” automatic magic
- treating drafts as permanent hidden internal state with no clear promotion path

## 3. System Overview

### 3.1 Main Components

1. `Creation Wizard`
   - drives multi-step artifact creation flows

2. `Preset Resolver`
   - applies compatibility presets and their constraints

3. `Draft Store`
   - stores local not-yet-promoted artifacts under internal draft collections

4. `Preview Renderer`
   - shows exact folder tree and emitted files before write

5. `Promotion Writer`
   - writes validated drafts into canonical repository contracts

6. `Import Flow`
   - reads and classifies existing skill folders

7. `Lineage Tracker`
   - records create, fork, rename, augment, and promotion provenance

### 3.2 Abstraction Levels

1. `Artifact Type Layer`
   - skill, deck, template, source metadata, import

2. `Preset Layer`
   - compatibility posture for emitted artifacts

3. `Draft Layer`
   - editable local artifact state under `internal:drafts` or `internal:collections`

4. `Preview Layer`
   - deterministic output projection

5. `Promotion Layer`
   - repository write or export action

### 3.3 External Dependencies

- repository contract rules from [05-REPOSITORY-CONTRACTS.md](/Users/axient/repository/praxis/specs/05-REPOSITORY-CONTRACTS.md)
- runtime slot rules from [04-RUNTIME-TARGET-PROFILES.md](/Users/axient/repository/praxis/specs/04-RUNTIME-TARGET-PROFILES.md)
- local filesystem access for drafts and previews

### 3.4 Project Structure and Key Paths

- [03-SPEC.md](/Users/axient/repository/praxis/specs/03-SPEC.md) - canonical product contract
- [05-REPOSITORY-CONTRACTS.md](/Users/axient/repository/praxis/specs/05-REPOSITORY-CONTRACTS.md) - emitted repository contract shapes
- [06-CREATION-SYSTEM.md](/Users/axient/repository/praxis/specs/06-CREATION-SYSTEM.md) - creation contract
- [07-UX-IA.md](/Users/axient/repository/praxis/specs/07-UX-IA.md) - Create surface behavior

## 4. Core Domain Model

### 4.1 Entities

#### 4.1.1 ArtifactType

Definition:
Kind of artifact the user is creating or importing.

Fields:

- `id` (`skill | deck | agent_file_template | source_metadata | import_skill_folder`)
- `display_name`

Lifecycle:
Chosen at wizard entry.

#### 4.1.2 CompatibilityPreset

Definition:
Ruleset that constrains emitted artifact shape.

Fields:

- `id`
- `artifact_types[]`
- `required_files[]`
- `optional_files[]`
- `validation_rules[]`

Lifecycle:
Chosen after artifact type and enforced through preview and promotion.

#### 4.1.3 DraftArtifact

Definition:
Local editable artifact not yet promoted to a repository contract.

Fields:

- `id`
- `artifact_type`
- `preset`
- `content`
- `metadata`
- `validation_state`

Lifecycle:
Created in the wizard, edited locally, then promoted or discarded. Lineage must preserve whether the draft came from create, import, fork, rename, or augment flow.

#### 4.1.4 PreviewTree

Definition:
Deterministic file-tree projection of a draft.

Fields:

- `artifact_id`
- `paths[]`
- `warnings[]`

Lifecycle:
Recomputed after every material draft change.

#### 4.1.5 PromotionTarget

Definition:
Destination for a validated draft.

Fields:

- `kind` (`repository_write | export_package`)
- `path`
- `conflicts[]`

Lifecycle:
Chosen at the final promotion step.

### 4.2 Stable Identifiers and Normalization Rules

- draft ids are stable within local draft storage
- preset ids are stable contracts, not UI labels
- preview paths must be deterministic for the same draft state
- promotion must preserve canonical filenames defined by repository contracts
- draft provenance must preserve source version or parent draft lineage when fork, rename, or augment is used

## 5. Domain Contract

### 5.1 Artifact and Preset Contract

Supported artifact types are:

- `skill`
- `deck`
- `agent_file_template`
- `source_metadata`
- `import_skill_folder`
- `fork_skill`
- `augment_skill`

Supported presets are:

- `open-standard-skill`
- `codex-enriched-skill`
- `claude-native-skill`
- `gemini-integration-ready`
- `multi-runtime-shared`

## 6. Configuration and Input Contract

### 6.1 Precedence and Defaults

Creation precedence is:

1. explicit artifact type
2. explicit preset
3. artifact metadata entered by the user
4. creation defaults from settings

### 6.2 Validation and Coercion

Validation must ensure:

- minimum required fields exist for the chosen artifact type
- emitted files match the selected preset
- preview tree is deterministic
- invalid drafts remain editable but are blocked from promotion

## 7. Lifecycle or State Model

### 7.1 States

1. `editing`
   - draft is being created or modified

2. `invalid`
   - required rules fail validation

3. `valid`
   - draft passes current validation

4. `promoted`
   - draft has been written to a repository or exported package

### 7.2 Transitions and Guards

- create draft -> `editing`
- fail validation -> `invalid`
- pass validation -> `valid`
- promote validated draft -> `promoted`
- update a promoted or valid draft -> back to `editing`

## 8. Primary Workflows and Reconciliation

### 8.1 Primary Flow

1. choose artifact type
2. choose compatibility preset
3. enter metadata and content
4. compute preview tree
5. validate draft
6. save as draft or promote to repository/export target

### 8.2 Failure or Retry Branches

- invalid drafts remain editable
- import validation failures block promotion but do not block inspection
- preview conflicts must be visible before write
- fork, rename, and augment must preserve lineage even when promotion is deferred

## 9. Storage, Ownership, and Safety Boundaries

### 9.1 Managed Locations

- local draft storage for unpromoted artifacts under `internal:drafts`
- manual deck storage under `internal:collections`
- repository write targets for promoted artifacts
- export package destinations when chosen

### 9.2 Destructive Boundaries

- Create must not overwrite repository files silently
- invalid drafts must not be promoted
- presets must not emit files outside their declared compatibility posture

## 10. Execution or Interface Contract

### 10.1 Invocation Contract

Create flows are exposed through:

- Create surface in desktop
- CLI creation commands for automation
- import and promotion subflows

Preview is mandatory before write.

## 11. External Integration Contract

### 11.1 Required Operations

- repository contract validation
- runtime slot lookup for agent-file templates
- filesystem write or export packaging

## 12. Context Packaging and Prompt Inputs

### 12.1 Required Context

Always provide:

- [03-SPEC.md](/Users/axient/repository/praxis/specs/03-SPEC.md)
- [05-REPOSITORY-CONTRACTS.md](/Users/axient/repository/praxis/specs/05-REPOSITORY-CONTRACTS.md)
- [06-CREATION-SYSTEM.md](/Users/axient/repository/praxis/specs/06-CREATION-SYSTEM.md)

### 12.2 Task-Specific Context

Only include:

- the artifact type being authored
- the preset rules being enforced
- runtime profile docs only when template slots are involved

### 12.3 Context Reduction Rules

- one artifact type or preset rule change per prompt when possible
- keep benchmark and distribution docs out of narrow creation tasks

## 13. Logging, Status, and Observability

### 13.1 Operator-Visible Signals

- current artifact type and preset
- validation state
- preview tree
- promotion conflicts

### 13.2 Logs and Traces

- promotion must emit the exact written or exported paths
- invalid drafts must report exact failing rules

## 14. Failure Model and Recovery Strategy

### 14.1 Failure Classes

- `draft_invalid`
- `preset_incompatible`
- `preview_conflict`
- `slot_unknown`
- `promotion_target_conflict`

### 14.2 Recovery Behavior

- invalid drafts remain editable
- preset incompatibility requires user correction before promotion
- repository conflicts require explicit user intervention

## 15. Safety, Boundaries, and Human Approval Policy

### 15.1 Always Do

- show exact output tree before write
- preserve deterministic preview rendering
- block promotion of invalid drafts
- preserve lineage metadata for create, fork, rename, and augment

### 15.2 Ask First

- writing into an existing repository tree with path conflicts
- changing the selected preset after substantial draft content already exists
- promoting templates into runtime slots with side effects outside the current scope
- converting Gemini integration presets into runtime-file install targets

### 15.3 Never Do

- never emit invalid artifacts for the selected preset
- never hide repository write conflicts
- never treat drafts as promoted artifacts without a promotion action

## 16. Reference Algorithms and Task Decomposition

### 16.1 Reference Algorithm

1. choose artifact type
2. choose preset
3. collect metadata and content
4. compute preview tree
5. validate draft
6. save, promote, or export

### 16.2 Task Units and Parallelism

- skill, deck, source metadata, and template creation are separate task families
- import validation should remain isolated from promotion write logic

## 17. Validation, Commands, and Success Criteria

### 17.1 Commands

- `praxis create skill`
- `praxis create deck`
- `praxis create template`
- `praxis create source-metadata`
- `praxis import skill-folder <path>`
- `praxis skill fork <skill-version-id>`
- `praxis skill augment <skill-version-id>`

### 17.2 Validation Matrix

- presets emit only allowed files
- previews show exact output trees
- invalid drafts remain editable but non-promotable
- imported skill folders are classified into compatible tiers
- fork and augment preserve parent lineage in draft metadata

### 17.3 Acceptance and Conformance Gates

- every create flow has deterministic preview
- every promotion path respects repository contracts
- no preset emits invalid artifacts by default

## 18. Implementation Checklist and Change Control

### 18.1 Required for Conformance

- artifact-type selection implemented
- preset selection implemented
- deterministic preview implemented
- local draft storage implemented
- promotion gating implemented
- lineage tracking for create, fork, rename, and augment implemented

### 18.2 Recommended Extensions

- artifact diff views
- draft history
- smarter preset recommendations

### 18.3 Spec Update Triggers

- new artifact types are added
- preset rules change
- repository contract shapes change
