# Praxis Repository Contracts Specification

Status: Draft v2

Purpose: Define how Praxis interprets source repositories, optional manifests, portable skills, decks, and agent-file templates while preserving open-standard usefulness.

## 1. Problem Statement

Praxis must work with generic repositories but also support richer repository contracts for better discovery, display, selection, and composition.

Important boundary:

- optional manifests enrich UX but must not be required for a source to be useful
- portable skills remain useful outside Praxis
- Praxis-specific metadata may enrich a repository without making its basic artifacts Praxis-locked

## 2. Goals and Non-Goals

### 2.1 Goals

- support portable open-standard sources
- support optional enriched repository manifests
- support declared decks and agent-file templates
- preserve unknown metadata keys where structured manifests are read and rewritten
- keep recipe-backed augmentation narrow and explicit

### 2.2 Non-Goals

- requiring Praxis-specific files for base discovery
- inferring arbitrary Markdown as runtime templates without declaration
- turning recipe-backed behavior into the default scanning mode

## 3. System Overview

### 3.1 Main Components

1. `Source Scanner`
   - finds candidate skill roots and manifests

2. `Manifest Reader`
   - loads optional structured repository sidecars

3. `Deck Resolver`
   - validates and normalizes declared decks

4. `Template Manifest Reader`
   - validates agent-file template declarations

5. `Recipe Adapter`
   - augments sources when generic discovery is insufficient

### 3.2 Abstraction Levels

1. `Portable Source Layer`
   - minimum open-standard skill repository

2. `Enriched Source Layer`
   - optional sidecars for richer UX

3. `Recipe Layer`
   - explicit augmentation for special shapes

4. `Contract Layer`
   - normalized repository-facing view used by planner and Library

### 3.3 External Dependencies

- source repository contents
- optional repository manifests
- optional recipe rules

### 3.4 Project Structure and Key Paths

- [03-SPEC.md](03-SPEC.md) - canonical product contract
- [05-REPOSITORY-CONTRACTS.md](05-REPOSITORY-CONTRACTS.md) - repository-facing contract
- [04-RUNTIME-TARGET-PROFILES.md](04-RUNTIME-TARGET-PROFILES.md) - slot ids referenced by templates
- [06-CREATION-SYSTEM.md](06-CREATION-SYSTEM.md) - creation flows that emit these manifests

## 4. Core Domain Model

### 4.1 Entities

#### 4.1.1 CompatibilityTier

Definition:
Classification of how much structured repository contract a source provides.

Fields:

- `id` (`portable | enriched | recipe`)
- `requirements[]`
- `behavior_notes[]`

Lifecycle:
Assigned during catalog compilation.

#### 4.1.2 SkillDirectory

Definition:
Directory that contains one portable skill.

Fields:

- `path`
- `skill_name`
- `description`
- `support_dirs[]`
- `sidecars[]`

Lifecycle:
Discovered by generic scanning or recipe augmentation.

#### 4.1.3 DeckManifest

Definition:
Optional declared deck manifest.

Fields:

- `version`
- `decks[]`

Lifecycle:
Read when present and validated against discovered skills.

#### 4.1.4 SourceManifest

Definition:
Optional repository metadata manifest.

Fields:

- `version`
- `display_name`
- `description`
- `recommended_target_profiles[]`
- `starter_tags[]`
- `recommended_decks[]`

Lifecycle:
Read when present for richer source display and defaults.

#### 4.1.5 AgentFileTemplateManifest

Definition:
Optional declared template manifest for runtime instruction blocks.

Fields:

- `version`
- `templates[]`

Lifecycle:
Read when present and validated against known slot ids.

#### 4.1.6 RecipeContract

Definition:
Explicit augmentation contract for sources that are not fully representable by generic rules.

Fields:

- `id`
- `scope`
- `augmentation_rules[]`

Lifecycle:
Applied only when recipe-backed handling is required.

### 4.2 Stable Identifiers and Normalization Rules

- portable skill identity is the `name` declared in `SKILL.md`
- deck ids are unique within one source
- template ids are unique within one manifest
- JSON manifests must include `version: 1` unless a narrower contract says otherwise
- unknown top-level keys must be preserved when recognized manifests are read and rewritten
- relative paths resolve from the repository root unless a manifest says otherwise

## 5. Domain Contract

### 5.1 Repository Compatibility Tiers

Portable tier:

- one or more directories containing `SKILL.md`
- valid `name` and `description`

Enriched tier may additionally include:

- `skill.json`
- `agents/openai.yaml`
- `skills.deck.json`
- `agent-files/manifest.json`
- `praxis.source.json`

Recipe tier:

- source requires explicit augmentation because generic repository scanning is insufficient

## 6. Configuration and Input Contract

### 6.1 Precedence and Defaults

Repository contract precedence is:

1. explicit recipe rules
2. explicit manifests
3. generic open-standard discovery

### 6.2 Validation and Coercion

Validation must check:

- every discovered skill contains `SKILL.md`
- required frontmatter `name` and `description` exist
- deck members reference discovered skills
- template slots reference known runtime slot ids
- manifest versions match the supported contract version

## 7. Lifecycle or State Model

### 7.1 States

1. `portable`
   - source is usable through generic open-standard discovery

2. `enriched`
   - source provides optional structured metadata

3. `recipe-backed`
   - source is usable only with explicit augmentation support

4. `invalid`
   - required portable or declared structured rules fail validation

### 7.2 Transitions and Guards

- scan generic source -> `portable` or `invalid`
- discover optional manifests -> `enriched`
- require explicit augmentation -> `recipe-backed`
- broken manifest or contract references -> `invalid`

## 8. Primary Workflows and Reconciliation

### 8.1 Primary Flow

1. scan candidate repository roots
2. discover portable skills
3. read optional manifests
4. validate decks, templates, and source metadata
5. apply recipe augmentation if required
6. emit normalized catalog and compatibility tier

### 8.2 Failure or Retry Branches

- invalid portable contracts block that source from being treated as a valid catalog
- invalid optional manifests may keep the source partially usable if the portable layer remains valid
- recipe failures block recipe-backed augmentation but should remain visible as specific errors

## 9. Storage, Ownership, and Safety Boundaries

### 9.1 Managed Locations

Recognized repository contract files are:

- `SKILL.md`
- `skill.json`
- `agents/openai.yaml`
- `skills.deck.json`
- `praxis.source.json`
- `agent-files/manifest.json`

### 9.2 Destructive Boundaries

- Praxis must not rewrite arbitrary repository files as if they were managed manifests
- undeclared Markdown files must not be treated as templates
- unknown keys in recognized manifests must not be discarded

## 10. Execution or Interface Contract

### 10.1 Invocation Contract

Repository contracts are consumed by:

- inspect
- plan
- apply
- create/import/promote
- doctor

Generic discovery remains the baseline execution path.

## 11. External Integration Contract

### 11.1 Required Operations

- local filesystem scanning
- optional GitHub source retrieval
- recipe lookup when the source requires augmentation

## 12. Context Packaging and Prompt Inputs

### 12.1 Required Context

Always provide:

- [03-SPEC.md](03-SPEC.md)
- [05-REPOSITORY-CONTRACTS.md](05-REPOSITORY-CONTRACTS.md)

### 12.2 Task-Specific Context

Only include:

- the manifest or contract type being modified
- the source examples relevant to the bug or feature
- runtime profile docs only if slot validation is involved

### 12.3 Context Reduction Rules

- one manifest contract change per prompt when possible
- do not include UX or distribution docs for narrow repository parser work

## 13. Logging, Status, and Observability

### 13.1 Operator-Visible Signals

- compatibility tier
- manifest validation errors
- deck and template reference errors
- recipe augmentation status

### 13.2 Logs and Traces

- inspect output should explain why a source was classified as portable, enriched, recipe-backed, or invalid
- doctor output should name the exact failing manifest or missing reference

## 14. Failure Model and Recovery Strategy

### 14.1 Failure Classes

- `skill_missing_required_frontmatter`
- `deck_member_unknown`
- `template_slot_unknown`
- `manifest_version_unsupported`
- `recipe_error`

### 14.2 Recovery Behavior

- invalid skills block portable discovery for that skill
- invalid decks or templates block those declared features but should not erase valid portable skills
- unsupported manifest versions require explicit upgrade support before promotion

## 15. Safety, Boundaries, and Human Approval Policy

### 15.1 Always Do

- preserve portable usefulness as the baseline
- keep optional manifest handling explicit
- preserve unknown keys in recognized JSON manifests

### 15.2 Ask First

- changing canonical manifest filenames
- changing manifest version semantics
- broadening recipe scope so far that generic scanning is bypassed by default

### 15.3 Never Do

- never require Praxis-only manifests for baseline source usefulness
- never infer arbitrary Markdown files as templates
- never drop unknown manifest keys during a read-rewrite cycle

## 16. Reference Algorithms and Task Decomposition

### 16.1 Reference Algorithm

1. scan roots for candidate skills
2. validate required portable fields
3. read optional manifests if present
4. validate declared cross-references
5. classify the source into a compatibility tier
6. augment with recipe logic only when required

### 16.2 Task Units and Parallelism

- skill validation, deck validation, and template manifest validation are separable review units
- recipe logic should remain isolated from the generic scanner

## 17. Validation, Commands, and Success Criteria

### 17.1 Commands

- `praxis inspect <source>` - discover repository contracts and compatibility tier
- `praxis doctor --source <source>` - report repository contract violations
- `praxis create source-metadata` - emit a valid `praxis.source.json` draft

### 17.2 Validation Matrix

- portable skills are discoverable with only `SKILL.md`
- enriched manifests improve metadata without being required
- deck members resolve to discovered skills
- template manifests reference only known slot ids
- recipe-backed sources remain explicit and narrow

### 17.3 Acceptance and Conformance Gates

- portable repositories remain usable without Praxis-specific files
- enriched repositories preserve unknown structured metadata keys
- invalid declared contracts are surfaced with exact errors rather than silent fallback

## 18. Implementation Checklist and Change Control

### 18.1 Required for Conformance

- portable skill discovery implemented
- optional deck manifest support implemented
- optional source metadata manifest support implemented
- optional template manifest support implemented
- compatibility tier classification implemented

### 18.2 Recommended Extensions

- richer source linting
- repository contract autofix suggestions
- migration helpers for future manifest versions

### 18.3 Spec Update Triggers

- manifest version changes
- new recognized repository sidecars are added
- recipe system becomes more than a narrow augmentation layer
