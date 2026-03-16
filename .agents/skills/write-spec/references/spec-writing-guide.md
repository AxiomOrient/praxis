# Contract-Style Spec Writing Guide

## Purpose

Use this guide when writing a `SPEC.md`-class document.

This style is not "write a long design doc".
It is "write an operational contract that another agent or engineer can implement against without guessing".

## What Makes a Spec "Contract-Style"

A contract-style spec has these properties:

1. It starts with boundary and problem, not architecture worship.
2. It distinguishes goals from non-goals early.
3. It defines the system in layers and named components before implementation details.
4. It names typed entities, ids, normalization rules, and state vocabulary explicitly.
5. It turns important files, manifests, APIs, or repository shapes into contracts.
6. It defines lifecycle or orchestration state instead of leaving behavior implicit.
7. It describes failure classes and recovery rules, not just happy-path behavior.
8. It includes validation and conformance criteria so "done" is testable.
9. It uses numbered sections and stable subsection names so readers can navigate and compare versions.

## What It Is Not

It is not:

- a product pitch
- a roadmap
- a feature brainstorm
- a loose architecture memo
- UI copywriting
- a changelog

If the document cannot answer "what are the exact states, contracts, failure modes, and validation gates?", it is not yet contract-style.

## Canonical Top-Level Shape

Default to this top-level shape:

1. Problem Statement
2. Goals and Non-Goals
3. System Overview
4. Core Domain Model
5. Domain Contract
6. Configuration or Input Contract
7. State Machine or Lifecycle Model
8. Core Workflows / Reconciliation / Scheduling
9. Storage / Workspace / Safety Boundary
10. Execution or Protocol Contract
11. External Integration Contract
12. Input / Request / Context Construction
13. Logging / Status / Observability
14. Failure Model and Recovery Strategy
15. Security and Operational Safety
16. Reference Algorithms
17. Test and Validation Matrix
18. Implementation Checklist

Do not obsess over title wording.
Do preserve the intent and sequence.
Optional appendices may follow section 18, but they are not part of the required contract skeleton.

Validation modes:

- `adapted` mode: for product, workflow, or repository specs that preserve the 18-section contract shape while renaming domain sections.
- `strict` mode: for service or orchestration specs that should stay close to the service-spec actual heading set.

## Section-by-Section Intent

### 1. Problem Statement

Answer:

- what the system is
- what problem it solves
- what boundary it does not cross

Include one "important boundary" block when the system is likely to be misunderstood.

### 2. Goals and Non-Goals

Goals are capabilities the system must satisfy.
Non-goals are explicit refusals that protect scope.

If a future reader could plausibly assume a capability exists, consider adding it as a non-goal.

### 3. System Overview

Describe:

- main components
- abstraction layers
- external dependencies

This is the zoomed-out map.
It should make later contracts unsurprising.

### 4. Core Domain Model

Define the nouns the rest of the document relies on.

Always include:

- entities
- key fields
- stable identifiers
- normalization rules

If ids are ambiguous, the spec is not ready.

### 5. Domain Contract

Turn the domain boundary into a contract.

Examples:

- workflow file contract
- repository contract
- source package contract
- manifest contract
- API object contract

This section explains what shape the outside world must present for the system to behave correctly.

### 6. Configuration or Input Contract

Explain:

- precedence rules
- defaults
- coercion
- validation gates
- reload semantics if applicable

If the system accepts input from more than one place, precedence must be explicit.

### 7. State Machine or Lifecycle Model

This is the biggest difference between vague specs and contract-style specs.

Define:

- internal states
- transition triggers
- idempotency or recovery invariants

If the system has no long-running orchestration, use lifecycle states instead of a live state machine.

### 8. Core Workflows / Reconciliation

Describe the central loops or workflows:

- scheduling
- install/apply/reconcile
- update/remove
- promotion/evaluation

Readers should be able to simulate a run from this section.

### 9. Storage / Workspace / Safety Boundary

Document where state lives and what safety invariants apply.

Examples:

- workspace layout
- manifest + lock storage
- target directories
- filesystem safety rules

### 10. Execution or Protocol Contract

Define how the system launches work or speaks to another subsystem.

Examples:

- runner launch contract
- app-server protocol
- planner / reconciler contract
- artifact renderer contract

### 11. External Integration Contract

Specify the outside systems the product depends on.

Examples:

- issue tracker adapter
- GitHub source resolver
- runtime-specific agent targets
- package manager integration

### 12. Input / Request / Context Construction

Explain how the system constructs the work payload.

Examples:

- prompt construction
- plan rendering inputs
- selection resolution
- template composition

### 13. Logging / Status / Observability

State:

- what operators or users can inspect
- what metrics or reports exist
- what minimum observability is required

### 14. Failure Model and Recovery Strategy

List failure classes, then say what recovery looks like.

Do not stop at enumerating errors.
Say whether the system retries, preserves state, blocks apply, or requires intervention.

### 15. Security and Operational Safety

Document trust assumptions and operational guardrails.

Examples:

- filesystem safety
- secret handling
- destructive action rules
- overwrite refusal rules

### 16. Reference Algorithms

Give language-agnostic step sequences for the most important flows.

This section is valuable because it collapses prose into executable reasoning without forcing code.

### 17. Test and Validation Matrix

State how conformance is verified.

Examples:

- parser tests
- reconciliation tests
- integration tests
- observability checks

### 18. Implementation Checklist

Close with a definition-of-done checklist.

Separate:

- required for conformance
- recommended extensions
- pre-production or release validation

## Writing Rules

### Use Contract Language

Prefer:

- `must`
- `should`
- `may`
- `required`
- `optional`
- `out of scope`

Avoid:

- `nice`
- `simple`
- `robust`
- `intuitive`
- `fast`

unless you define what those words mean operationally.

### Define Stable Nouns

Pick one noun for each concept and do not drift.

Bad:

- source / repository / package / import used interchangeably

Good:

- `SourceRef` is the canonical noun
- `Catalog` is the scanned result
- `Plan` is the pure preview

### Make Determinism Visible

If ordering matters, write the sort order.
If precedence matters, write the precedence.
If repair is possible, write when it is safe.

### Preserve Important Boundaries

State clearly what the system does not own.

Standard specs do this repeatedly:

- it does not define business logic inside the orchestrator
- it does not mandate one UI
- it does not prescribe one security posture

Carry that discipline into product specs.

### Keep Top-Level Sections Stable

Readers should be able to diff two versions of the spec without re-learning the structure.
Use the same section numbers across versions whenever possible.

## Adaptation Heuristics

### For Product Specs

- keep the 18-section skeleton
- adapt section 5 to repository/source/content contracts
- adapt section 7 to lifecycle states instead of worker orchestration states
- adapt section 10 to planner/reconciler/composer contracts

### For Service Specs

- keep the service names almost exactly
- include explicit runtime state and reconciliation loops

### For Strict Mirroring

Use strict mirroring when the target system is itself an orchestrator, worker service, scheduler, or runner.

In that case, prefer standard actual section names for:

- Workflow Specification (Repository Contract)
- Configuration Specification
- Orchestration State Machine
- Polling, Scheduling, and Reconciliation
- Workspace Management and Safety
- Agent Runner Protocol
- Issue Tracker Integration Contract
- Prompt Construction and Context Assembly
- Logging, Status, and Observability
- Failure Model and Recovery Strategy
- Security and Operational Safety
- Reference Algorithms
- Test and Validation Matrix
- Implementation Checklist

### For Workflow Specs

- treat the workflow file or prompt contract as section 5
- treat dynamic config and trigger rules as section 6
- use section 16 for reference algorithms or decision trees

## Review Checklist

Before calling a spec complete, verify all are true:

- the top-level numbered structure is present and continuous
- the problem statement includes an important boundary
- non-goals remove likely scope ambiguity
- entities, ids, and normalization rules are explicit
- a contract section exists for every external or file-based dependency that matters
- lifecycle or orchestration state is explicit
- failure classes and recovery rules are both present
- at least one validation matrix and one checklist section exist
- vague product language has been replaced with operational rules

## Common Failure Modes

### Architecture Without Contracts

Symptoms:

- strong component diagrams
- weak file/API/schema contracts

Fix:

- turn every important boundary into an explicit contract section

### Feature List Without States

Symptoms:

- many capabilities
- no lifecycle or transition rules

Fix:

- define the internal states and transition triggers

### Error List Without Recovery

Symptoms:

- section 14 is just bullet points of exceptions

Fix:

- add retry, preservation, blocking, repair, and intervention behavior

### Checklist Without Validation

Symptoms:

- "Definition of done" is hand-wavy

Fix:

- create a validation matrix before the checklist
