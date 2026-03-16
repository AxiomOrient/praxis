# PRAXIS Spec Writing Standard

Status: Working standard v2

Purpose: Define how to write AI-facing specifications that are precise enough to guide autonomous coding work, safe enough to constrain risky behavior, and structured enough to support planning, task breakdown, implementation, and review.

Primary use:

- writing new canonical specs
- upgrading loose notes into implementable specs
- reviewing whether an existing spec is actionable for an AI coding agent
- defining repository-local standards for skills, plans, and execution workflows

## 1. Why This Standard Exists

A good AI-facing spec is not a memo.
It is an execution contract.

It must do four jobs at once:

1. explain what is being built and why
2. define the contracts the system must obey
3. constrain what the agent may, should, and must not do
4. make success and failure observable through commands, tests, and conformance checks

This standard combines:

- the strong contract structure already present in Praxis `write-spec`
- the AI-agent spec guidance distilled from `aiagentspec.pdf`
- spec-driven workflow guidance from GitHub Spec Kit
- prompt and task-structuring guidance from Anthropic docs
- prompt and skill-structure guidance from Harness AI docs

## 2. The Working Model

Do not treat “the spec” as a single blob.
Treat it as a layered artifact set.

### 2.1 Layer A: Product Brief

This is the short, high-level description.
It answers:

- who is the user or operator
- what problem exists
- why this capability matters
- what success looks like
- what is out of scope

This layer should be brief and directional.

### 2.2 Layer B: Contract Spec

This is the canonical implementation contract.
It defines entities, files, states, workflows, failures, interfaces, and validation.

This is where ambiguity must be removed.

### 2.3 Layer C: Plan

The plan turns the contract into a technical strategy:

- architecture choices
- stack constraints
- sequencing
- tradeoffs
- migration approach

### 2.4 Layer D: Tasks

Tasks split the plan into reviewable, testable units.

Each task should:

- solve one bounded problem
- have clear inputs and outputs
- be testable in isolation
- avoid unrelated context

### 2.5 Layer E: Implementation and Review

Execution happens only after the previous layers are good enough.
Specs are living documents and must be updated when reality changes.

## 3. Core Principles

1. Facts and decisions must be separable.
2. The spec must reflect current code reality or explicitly declare intended divergence.
3. Vague wording is not allowed.
4. Boundaries must be written as contracts, not implied by tone.
5. Defaults, failures, recovery, and migration rules must be explicit.
6. A spec must be testable before it is called complete.
7. Large tasks must be decomposed into smaller prompts or work units.
8. The spec must be usable by both humans and AI agents.

## 4. Evidence Rules

Use this evidence priority when sources conflict:

1. current running code
2. current canonical spec
3. current implementation plan or task ledger
4. other repository docs
5. examples, demos, or screenshots
6. comments, UI copy, or implied behavior

Rules:

- Never infer backend contracts from UI wording alone.
- Never finalize a product contract from examples alone.
- If code and spec differ, classify the mismatch explicitly:
  - current bug
  - outdated doc
  - intentional future divergence

## 5. Mandatory Six Operational Areas

Every AI-facing spec must cover these six operational areas either in the main body or in a dedicated execution appendix.

### 5.1 Commands

List exact executable commands.

- include full command lines
- include flags when relevant
- note whether commands mutate state

Examples:

- `npm test`
- `pytest -v`
- `cargo test --workspace`

### 5.2 Testing

State how correctness is verified.

- test framework
- command
- test location
- required pass criteria
- conformance cases if any

### 5.3 Project Structure

State where important things live.

- source directories
- test directories
- docs directories
- generated output
- config and manifest locations

### 5.4 Code Style

Do not describe style abstractly if examples can make it concrete.

- naming rules
- formatting rules
- preferred patterns
- prohibited patterns

### 5.5 Git Workflow

Document workflow expectations when relevant:

- branch naming
- commit message format
- PR expectations
- rebase, merge, or squash rules

### 5.6 Boundaries

Document what the agent may and may not touch.

- secrets
- generated directories
- vendored dependencies
- production config
- infrastructure definitions
- other high-blast-radius files

## 6. Three-Tier Boundary Policy

Every AI-facing spec should express boundary rules using three levels.

### 6.1 Always Do

Actions that should happen without asking.

Examples:

- always run validation before handoff
- always follow naming conventions
- always preserve structured logs

### 6.2 Ask First

Actions that might be valid but need approval.

Examples:

- schema changes
- dependency additions
- CI/CD changes
- deleting tests

### 6.3 Never Do

Hard stops.

Examples:

- never commit secrets
- never edit vendored dependencies
- never remove failing tests without approval

## 7. Required Spec Shape

For full implementation specs, use this 18-section top-level skeleton.

1. Problem Statement
2. Goals and Non-Goals
3. System Overview
4. Core Domain Model
5. Domain Contract
6. Configuration and Input Contract
7. Lifecycle or State Model
8. Primary Workflows and Reconciliation
9. Storage, Ownership, and Safety Boundaries
10. Execution or Interface Contract
11. External Integration Contract
12. Context Packaging and Prompt Inputs
13. Logging, Status, and Observability
14. Failure Model and Recovery Strategy
15. Safety, Boundaries, and Human Approval Policy
16. Reference Algorithms and Task Decomposition
17. Validation, Commands, and Success Criteria
18. Implementation Checklist and Change Control

Optional appendices may follow section 18.

## 8. Section Intent

### 8.1 Problem Statement

Must answer:

- what the system is
- what problem it solves
- who uses it
- what boundary it does not cross

Include an “important boundary” block when confusion is likely.

### 8.2 Goals and Non-Goals

Goals define required capability.
Non-goals remove likely ambiguity.

If a future implementer could reasonably assume something is included, either make it a goal or a non-goal.

### 8.3 System Overview

Define:

- main components
- abstraction levels
- external dependencies
- key repository paths

### 8.4 Core Domain Model

Define the stable nouns.

Every major entity should include:

- identity
- required fields
- optional fields
- mutability
- storage location
- lifecycle
- references and relations

### 8.5 Domain Contract

Turn the outside world into a contract.

Examples:

- repository contract
- workflow file contract
- manifest contract
- package contract
- API object contract

### 8.6 Configuration and Input Contract

Define:

- precedence
- defaults
- coercion
- validation
- reload behavior

### 8.7 Lifecycle or State Model

Make transitions explicit.

Must include:

- states
- entry conditions
- transition triggers
- guards
- terminal states

### 8.8 Primary Workflows and Reconciliation

Describe the core loops or workflows as stepwise logic:

- trigger
- inputs
- preconditions
- algorithm
- side effects
- failure modes
- idempotency

### 8.9 Storage, Ownership, and Safety Boundaries

Define where state lives and who owns it.

Must cover:

- file layout
- ownership and lock rules
- managed vs unmanaged paths
- destructive boundaries

### 8.10 Execution or Interface Contract

Define how work is launched or invoked:

- CLI
- API
- pipeline step
- app-server protocol
- renderer contract

### 8.11 External Integration Contract

Define external systems explicitly.

For each integration:

- required inputs
- output shape
- assumptions
- failure handling

### 8.12 Context Packaging and Prompt Inputs

This section is required for AI-facing specs.

It must state:

- what context is always required
- what context is task-specific
- what should be summarized vs fully included
- how large specs should be split
- when a fresh context or new session is preferable

### 8.13 Logging, Status, and Observability

Define:

- operator-visible state
- logs and traces
- status surfaces
- minimum observability requirements

### 8.14 Failure Model and Recovery Strategy

Define stable failure classes.

For each important failure:

- error code
- trigger
- severity
- what it blocks
- recovery path

### 8.15 Safety, Boundaries, and Human Approval Policy

Include:

- Always Do
- Ask First
- Never Do
- trust assumptions
- secret and destructive-action policy

### 8.16 Reference Algorithms and Task Decomposition

Give language-agnostic step sequences for important flows.

Also define:

- how work breaks into tasks
- what can run in parallel
- what must remain sequential
- what boundaries prevent task collisions

### 8.17 Validation, Commands, and Success Criteria

This section must make “done” testable.

Include:

- exact commands
- validation matrix
- success criteria
- conformance or regression suite rules
- measurable gates where possible

### 8.18 Implementation Checklist and Change Control

Close with:

- required for conformance
- recommended extensions
- operational validation before release
- when the spec must be updated

## 9. Sentence-Level Writing Rules

### 9.1 Prohibited Phrases

Do not use empty language like:

- appropriately
- if needed
- flexibly
- efficiently
- safely handles
- user-friendly
- enough

Replace vague claims with observable rules.

### 9.2 Normative Wording

Use these intentionally:

- `MUST`
- `MUST NOT`
- `SHOULD`
- `MAY`

If a rule is mandatory, write it as a concrete obligation.

### 9.3 Stable Terms

Choose one term for each concept and reuse it consistently.

If the document alternates between `workspace`, `project dir`, and `target path` for the same thing, the spec is not ready.

## 10. Required Entity Rules

When introducing an entity, define:

1. name
2. identity
3. id rules
4. mutability
5. stored location
6. required fields
7. optional fields
8. relationships
9. lifecycle
10. deletion or archival behavior
11. migration impact if relevant

## 11. Required Workflow Rules

Every important workflow should specify:

1. trigger
2. inputs
3. preconditions
4. algorithm
5. side effects
6. failure modes
7. idempotency
8. output or result object

## 12. Required File-Format Rules

For every external or persisted file format, specify:

1. path or location
2. file format
3. version field
4. minimum example
5. required fields
6. defaults
7. unknown-field behavior
8. compatibility and migration rules

## 13. Defaults, Migration, and Drift

If there is a default, specify:

1. the default value
2. why it exists
3. how it can be overridden
4. override precedence

If current code and target design diverge, include a migration strategy:

- from
- to
- detection
- read strategy
- write strategy
- rollback or safety guarantees

## 14. Validation Standard

Specs must define both structural and behavioral validation.

### 14.1 Structural Validation

The document should have:

- stable top-level shape
- stable section naming
- deterministic subsection order

### 14.2 Behavioral Validation

The system should have:

- test cases
- conformance rules
- acceptance criteria
- human review gates for subjective quality

### 14.3 Success Criteria

Success criteria should be:

- specific
- measurable
- achievable
- relevant

Whenever possible, prefer:

- pass/fail commands
- explicit thresholds
- edge-case handling expectations

## 15. Context Packaging Rules

Large context degrades quality.
Do not hand an agent the entire world by default.

Preferred pattern:

1. brief
2. contract spec
3. plan
4. task slice
5. relevant local code or files only

For large specs:

- create a concise summary or extended TOC
- keep a high-level overview available
- provide detailed sections on demand

Use one focused prompt or task per work unit whenever possible.

## 16. Minimal vs Full Specs

Do not force a heavyweight spec onto trivial work.

Use a minimal spec when:

- the task is isolated
- the blast radius is low
- no persistent state or security boundary is involved

Use a full 18-section spec when:

- state exists
- failures matter
- multiple systems integrate
- migration or persistence exists
- more than one contributor or agent will rely on the artifact

## 17. Review Loop Standard

Every serious spec workflow should include:

1. draft
2. review against criteria
3. refine
4. validate
5. execute
6. re-sync spec if reality changes

Recommended self-correction loop:

1. generate draft
2. review draft against explicit checklist
3. revise draft
4. validate structure
5. use for planning or implementation

## 18. Practical Checklist

Before handing a spec to an AI coding agent, verify:

- the mission is explicit
- non-goals remove ambiguity
- the six operational areas are covered
- Always / Ask First / Never policy exists
- current evidence is cited or known
- entities, ids, states, and files are explicit
- failures and recovery rules are explicit
- commands and validation gates are explicit
- the spec can be split into task-sized contexts
- the spec can be updated as reality changes

## 19. Praxis-Specific Expectations

For Praxis repository work:

- do not ignore current code reality
- do not infer contracts from UI copy
- do not omit migration where code and target design diverge
- do not ship schema, CLI, job, benchmark, or UI specs without validation rules
- use `OPEN` only for genuinely unresolved decisions

`OPEN` entries should include:

- what is unresolved
- why it is unresolved
- what is already known
- when the decision must be made

## 20. Source Notes

This standard is synthesized from the following source material:

- Addy Osmani, “How to write a good spec for AI agents”
  - focus on high-level brief first, six operational areas, three-tier boundaries, modular task context, self-checks, and iterative spec updates
  - [https://addyosmani.com/blog/good-spec/](https://addyosmani.com/blog/good-spec/)
- GitHub Spec Kit
  - spec-driven workflow with constitution → specify → plan → tasks → implement
  - [https://github.com/github/spec-kit](https://github.com/github/spec-kit)
- Anthropic docs
  - clarity and directness, prompt chaining, self-checking, success criteria, context packaging
  - [https://docs.anthropic.com/en/docs/prompt-engineering](https://docs.anthropic.com/en/docs/prompt-engineering)
  - [https://platform.claude.com/docs/en/docs/build-with-claude/prompt-engineering/chain-prompts](https://platform.claude.com/docs/en/docs/build-with-claude/prompt-engineering/chain-prompts)
- Harness docs
  - clear detailed prompts, one resource per prompt, iterative refinement, skill anatomy, on-demand references
  - [https://developer.harness.io/docs/platform/harness-aida/effective-prompting-ai/](https://developer.harness.io/docs/platform/harness-aida/effective-prompting-ai/)
  - [https://developer.harness.io/docs/platform/harness-ai/harness-skills/](https://developer.harness.io/docs/platform/harness-ai/harness-skills/)
- OpenAI Symphony
  - contract-style service spec shape with explicit state, recovery, validation, and implementation checklist sections
  - [https://github.com/openai/symphony](https://github.com/openai/symphony)
- Praxis local `write-spec`
  - strong contract structure, 18-section discipline, template-first workflow, and structural validation script
