# Praxis Runtime Target Profiles Specification

Status: Draft v2

Purpose: Define how Praxis maps abstract artifacts into concrete runtime targets for Codex and Claude Code while keeping Gemini integration explicit but outside first-class runtime-file install targets.

## 1. Problem Statement

Praxis manages portable artifacts, but those artifacts land in runtimes with different skill roots, instruction files, layering rules, and supported scopes.

Important boundary:

- Praxis provides one abstract artifact model and explicit target profiles.
- Praxis does not claim that Codex, Claude Code, and Gemini CLI are semantically identical.
- Vendor-native semantics remain authoritative where documented contracts differ.

## 2. Goals and Non-Goals

### 2.1 Goals

- define canonical runtime identifiers
- define canonical slot identifiers and skill roots for first-class runtime-file targets
- support explicit target profiles for Codex and Claude Code
- preserve runtime-native instruction-file and root semantics
- allow shared roots only when runtime contracts genuinely overlap
- give planner and reconciler stable mapping inputs

### 2.2 Non-Goals

- pretending all runtimes use one universal instruction model
- full management of every vendor-native config file
- undocumented alias discovery as a first-class contract
- plugin, hook, or marketplace management as part of v1 runtime mapping
- declaring Gemini runtime-file roots or slots before a concrete contract exists

## 3. System Overview

### 3.1 Main Components

1. `Runtime Registry`
   - defines supported runtime ids

2. `Slot Registry`
   - defines canonical runtime instruction slots

3. `Skill Root Registry`
   - defines supported managed skill roots per runtime and scope

4. `Target Profile Resolver`
   - maps a selected profile into runtime roots and slots

5. `Compatibility Checker`
   - validates whether the chosen profile is compatible with the requested operation

### 3.2 Abstraction Levels

1. `Runtime Identity Layer`
   - runtime ids and native product names

2. `Slot Layer`
   - canonical instruction-file destinations

3. `Root Layer`
   - canonical skill installation destinations

4. `Profile Layer`
   - reusable multi-runtime mapping sets

5. `Validation Layer`
   - compatibility checks and blocked combinations

### 3.3 External Dependencies

- documented Codex runtime file and skill-root contracts
- documented Claude Code runtime file and skill-root contracts
- any future documented Gemini runtime file contract, if adopted

### 3.4 Project Structure and Key Paths

- [03-SPEC.md](03-SPEC.md) - canonical product contract
- [04-RUNTIME-TARGET-PROFILES.md](04-RUNTIME-TARGET-PROFILES.md) - runtime mapping contract
- [07-UX-IA.md](07-UX-IA.md) - UI surfaces that expose runtime targets
- [06-CREATION-SYSTEM.md](06-CREATION-SYSTEM.md) - presets that emit runtime-aware artifacts

## 4. Core Domain Model

### 4.1 Entities

#### 4.1.1 RuntimeId

Definition:
Stable identifier for one supported runtime.

Fields:

- `id` (`codex | claude | gemini`)
- `display_name`

Lifecycle:
Declared in the runtime registry and used by profiles, plans, and UI surfaces. Only `codex` and `claude` are currently first-class runtime-file install targets.

#### 4.1.2 SlotId

Definition:
Stable identifier for one managed runtime instruction destination.

Fields:

- `id`
- `runtime_id`
- `path_pattern`
- `scope_kind`

Lifecycle:
Used by template manifests, planner output, and agent-file composition.

#### 4.1.3 SkillRoot

Definition:
Managed installation root for one runtime and scope.

Fields:

- `runtime_id`
- `scope_kind`
- `path_pattern`
- `root_kind` (`native | shared`)

Lifecycle:
Resolved by target profile selection.

#### 4.1.4 TargetProfile

Definition:
Reusable runtime mapping policy.

Fields:

- `id`
- `agents[]`
- `skill_roots[]`
- `agent_file_slots[]`
- `shared_roots?`

Lifecycle:
Selected by the user or default settings and validated before planning.

#### 4.1.5 RuntimeTargetReport

Definition:
Validation result for one chosen profile and scope.

Fields:

- `target_profile`
- `resolved_roots[]`
- `resolved_slots[]`
- `warnings[]`
- `errors[]`

Lifecycle:
Produced during plan or doctor operations.

### 4.2 Stable Identifiers and Normalization Rules

- runtime ids are globally stable and lowercase
- slot ids are globally stable and must not be inferred from display labels
- root kinds are explicit rather than implied from path shape
- target profile ids are stable repository contracts
- undocumented aliases are out of scope until promoted into the documented set

## 5. Domain Contract

### 5.1 Runtime and Slot Contract

Supported runtime ids are:

- `codex`
- `claude`
- `gemini`

Current first-class runtime-file install targets are:

- `codex`
- `claude`

Gemini is currently a connection, creator, and benchmark integration target. It is not yet a first-class runtime-file install target.

Supported first-class slots are:

- `codex-user-root`
- `codex-user-override`
- `codex-project-root`
- `codex-project-override`
- `claude-user-root`
- `claude-project-root`
- `claude-project-dot`

Supported skill roots are:

- Codex repo: `$REPO_ROOT/.agents/skills`
- Codex user: `$HOME/.agents/skills`
- Claude repo: `$REPO_ROOT/.claude/skills`
- Claude user: `$HOME/.claude/skills`

## 6. Configuration and Input Contract

### 6.1 Precedence and Defaults

Runtime target selection precedence is:

1. explicit target profile passed by the user
2. explicit runtime-slot or root selection in the current surface
3. configured default target profile
4. product default profile

### 6.2 Validation and Coercion

Validation must reject at minimum:

- unknown runtime ids
- unknown slot ids
- profile-to-runtime combinations that reference incompatible slots
- shared roots where the underlying runtime contracts do not overlap
- any attempt to treat Gemini runtime-file roots or slots as first-class before the contract is explicitly adopted

## 7. Lifecycle or State Model

### 7.1 States

1. `unresolved`
   - target profile not yet expanded into concrete roots and slots

2. `resolved`
   - roots and slots expanded successfully

3. `incompatible`
   - one or more chosen roots or slots violate runtime constraints

### 7.2 Transitions and Guards

- choose target profile -> `unresolved`
- validate profile and scope -> `resolved` or `incompatible`
- change runtime docs or slot registry -> re-evaluate previous `resolved` mappings

## 8. Primary Workflows and Reconciliation

### 8.1 Primary Flow

1. read requested target profile
2. expand runtime ids, roots, and slots
3. validate profile against runtime-specific rules
4. surface exact destinations to planner and UI
5. block apply if the mapping is incompatible

### 8.2 Failure or Retry Branches

- incompatible profiles block plan/apply until corrected
- newly documented runtime changes require profile revalidation
- missing runtime roots may be reported as warnings if the mapping contract itself is still valid

## 9. Storage, Ownership, and Safety Boundaries

### 9.1 Managed Locations

- Codex manages `.agents/skills` and `AGENTS.md` slots
- Claude manages `.claude/skills` and `CLAUDE.md` slots
- Gemini connection or creator state may be observed, but Gemini runtime-file roots and slots are not managed by this contract

### 9.2 Destructive Boundaries

- Praxis must not write outside resolved managed roots or slots
- shared roots are not first-class in the final current boundary
- Praxis must not write Claude artifacts into shared `.agents/skills` roots
- Praxis must not invent Gemini runtime-file destinations

## 10. Execution or Interface Contract

### 10.1 Invocation Contract

Runtime mapping is used by:

- inspect and plan previews
- apply and remove flows
- agent-file composition
- doctor and compatibility reporting

The desktop app may present friendly labels, but CLI/core ids remain authoritative.

## 11. External Integration Contract

### 11.1 Required Operations

- Codex root and slot resolution
- Claude Code root and slot resolution
- compatibility checks against documented runtime contracts
- explicit surfacing that Gemini is integration-only until a runtime-file contract exists

## 12. Context Packaging and Prompt Inputs

### 12.1 Required Context

Always provide:

- [03-SPEC.md](03-SPEC.md)
- [04-RUNTIME-TARGET-PROFILES.md](04-RUNTIME-TARGET-PROFILES.md)

### 12.2 Task-Specific Context

Only include:

- the specific runtime contract being changed
- the planner or agent-file code paths involved
- the relevant UI surface if the task is presentation-oriented

### 12.3 Context Reduction Rules

- one runtime mapping change per prompt whenever possible
- do not include unrelated creation or benchmark docs for narrow runtime tasks

## 13. Logging, Status, and Observability

### 13.1 Operator-Visible Signals

- chosen target profile
- resolved roots and slots
- incompatible mapping errors
- warnings for unsupported runtime-target or slot assumptions

### 13.2 Logs and Traces

- plan and doctor output should show the exact runtime ids, roots, and slots used
- UI surfaces must expose what runtime files will change before apply

## 14. Failure Model and Recovery Strategy

### 14.1 Failure Classes

- `unknown_runtime`
- `unknown_slot`
- `target_profile_invalid`
- `runtime_contract_mismatch`
- `runtime_target_not_supported`

### 14.2 Recovery Behavior

- unknown runtimes and slots block plan and apply
- incompatible profiles require explicit user correction
- runtime contract mismatches remain visible in doctor output until mapping rules are updated
- unsupported Gemini runtime-file targets remain visible as scope-boundary errors rather than being silently coerced

## 15. Safety, Boundaries, and Human Approval Policy

### 15.1 Always Do

- preserve runtime-native semantics in every mapping decision
- show exact roots and slot paths before mutation
- keep unsupported runtime targets explicit rather than inferred

### 15.2 Ask First

- adding a new first-class runtime
- changing canonical slot ids
- changing the default multi-runtime profile
- promoting Gemini into the runtime-file install boundary

### 15.3 Never Do

- never pretend undocumented aliases are first-class slots
- never invent shared-root semantics that are not part of the adopted contract
- never treat Gemini as a runtime-file target before a concrete contract exists
- never write outside resolved managed roots and slots

## 16. Reference Algorithms and Task Decomposition

### 16.1 Reference Algorithm

1. load requested target profile
2. expand runtimes, roots, and slots
3. validate slot compatibility and current runtime-target support
4. emit resolved mapping report
5. pass resolved mapping into planner or doctor

### 16.2 Task Units and Parallelism

- Codex and Claude adapter work may be reasoned about independently
- Gemini integration work must be separated from runtime-file install contract work
- changes to slot ids, target-boundary logic, and default target profiles must be treated as separate review units

## 17. Validation, Commands, and Success Criteria

### 17.1 Commands

- `praxis plan --target-profile <id>` - preview resolved runtime mapping
- `praxis doctor --target-profile <id>` - report runtime-target incompatibilities
- `praxis agent-files preview --runtime <id> --slot <id>` - preview slot composition consequences

### 17.2 Validation Matrix

- Codex mappings preserve `AGENTS.md` layering and `.agents/skills`
- Claude mappings preserve `CLAUDE.md` slots and `.claude/skills`
- Gemini runtime-file mappings are rejected until a concrete contract is adopted
- unsupported runtime-file targets are surfaced explicitly before mutation

### 17.3 Acceptance and Conformance Gates

- every first-class runtime id, slot id, and root contract defined here is resolvable without ambiguity
- incompatible profiles are blocked before mutation
- plan and doctor surfaces show exact resolved destinations

## 18. Implementation Checklist and Change Control

### 18.1 Required for Conformance

- canonical runtime ids implemented
- canonical slot registry implemented
- canonical skill root registry implemented
- target profile validation implemented
- current runtime-target boundary checks implemented

### 18.2 Recommended Extensions

- richer runtime diagnostics
- runtime documentation freshness checks
- Gemini runtime-file support after a concrete contract is documented

### 18.3 Spec Update Triggers

- runtime documentation changes
- new slot support is added
- runtime-target boundary policy changes
- a new supported runtime is introduced
- Gemini runtime-file support becomes concrete enough to standardize
