# Praxis Distribution and Release Boundary Specification

Status: Draft v2

Purpose: Define the release forms, distribution channels, and release-safety boundary for Praxis.

## 1. Problem Statement

Praxis needs a release model that preserves inspectability, deterministic install semantics, and contract consistency across source, CLI, and desktop forms.

Important boundary:

- distribution is a packaging and release concern
- it is not a hosted control plane, registry backend, or mandatory cloud sync system

## 2. Goals and Non-Goals

### 2.1 Goals

- ship Praxis as source package, CLI binary, and macOS desktop app
- preserve the same core contracts across those forms
- use GitHub Releases as the primary distribution channel
- keep source auditability and deterministic install semantics intact

### 2.2 Non-Goals

- mandatory hosted services
- registry backend as a core dependency
- in-app marketplace service as part of v1 release boundary
- auto-updater as a first-release requirement

## 3. System Overview

### 3.1 Main Components

1. `Source Package`
   - auditable repository form

2. `CLI Binary`
   - terminal and scripting form

3. `macOS App Bundle`
   - visual desktop form

4. `Release Channel`
   - GitHub Releases distribution point

### 3.2 Abstraction Levels

1. `Source Layer`
   - repository and inspectable code

2. `CLI Layer`
   - automation and terminal operations

3. `Desktop Layer`
   - visual shell over the same contracts

4. `Release Layer`
   - packaging and publication

### 3.3 External Dependencies

- GitHub Releases
- local platform packaging for source, CLI, and macOS desktop outputs
- repository build scripts

### 3.4 Project Structure and Key Paths

- [08-DISTRIBUTION.md](/Users/axient/repository/praxis/specs/08-DISTRIBUTION.md) - release boundary contract
- [03-SPEC.md](/Users/axient/repository/praxis/specs/03-SPEC.md) - canonical product contract
- [scripts/package-source.sh](/Users/axient/repository/praxis/scripts/package-source.sh) - source packaging entry
- [scripts/release-cli.sh](/Users/axient/repository/praxis/scripts/release-cli.sh) - CLI release entry
- [scripts/release-macos.sh](/Users/axient/repository/praxis/scripts/release-macos.sh) - macOS release entry

## 4. Core Domain Model

### 4.1 Entities

#### 4.1.1 ReleaseArtifact

Definition:
One distributable Praxis artifact.

Fields:

- `kind` (`source | cli | macos_app`)
- `version`
- `path`

Lifecycle:
Built, validated, and published through release flow.

#### 4.1.2 ReleaseChannel

Definition:
Distribution endpoint for release artifacts.

Fields:

- `id`
- `kind`
- `artifacts[]`

Lifecycle:
Used during publish.

#### 4.1.3 PlatformTarget

Definition:
Platform-specific packaging target.

Fields:

- `id`
- `artifact_kinds[]`

Lifecycle:
Resolved during build and release scripts.

### 4.2 Stable Identifiers and Normalization Rules

- release artifact kinds are stable: `source`, `cli`, `macos_app`
- source package, CLI binary, and desktop app all describe the same canonical product contracts
- GitHub Releases is the primary channel until a different release contract is explicitly adopted

## 5. Domain Contract

### 5.1 Release Shape Contract

Praxis ships in three forms:

- source package
- CLI binary
- macOS desktop app

This is the complete core release shape.

## 6. Configuration and Input Contract

### 6.1 Precedence and Defaults

Release precedence is:

1. explicit release command or script
2. artifact-specific packaging rules
3. repository defaults

### 6.2 Validation and Coercion

Release validation must ensure:

- source remains inspectable
- CLI and desktop map to the same core contracts
- packaged artifacts match the requested release kind

## 7. Lifecycle or State Model

### 7.1 States

1. `draft`
   - artifact not yet packaged

2. `built`
   - artifact packaged locally

3. `validated`
   - artifact passes release checks

4. `published`
   - artifact attached to the release channel

### 7.2 Transitions and Guards

- build artifact -> `built`
- validate artifact -> `validated`
- publish to GitHub Releases -> `published`
- failed validation returns artifact to a corrective build stage

## 8. Primary Workflows and Reconciliation

### 8.1 Primary Flow

1. choose release artifact kind
2. run packaging script
3. validate output
4. publish to GitHub Releases

### 8.2 Failure or Retry Branches

- failed packaging blocks publication
- failed validation requires rebuild or fix before publication
- missing one artifact kind must not imply the other release kinds are valid

## 9. Storage, Ownership, and Safety Boundaries

### 9.1 Managed Locations

- source archive outputs
- CLI release outputs
- macOS bundle outputs

### 9.2 Destructive Boundaries

- distribution must not mutate product contracts silently
- packaging scripts must not depend on mandatory cloud infrastructure
- published artifacts must remain traceable to the source release state

## 10. Execution or Interface Contract

### 10.1 Invocation Contract

Release operations are invoked through repository packaging scripts and release workflows.

- source packaging uses `scripts/package-source.sh`
- CLI release uses `scripts/release-cli.sh`
- macOS release uses `scripts/release-macos.sh`

## 11. External Integration Contract

### 11.1 Required Operations

- GitHub Releases publication
- local build and packaging execution
- platform-specific artifact validation

## 12. Context Packaging and Prompt Inputs

### 12.1 Required Context

Always provide:

- [03-SPEC.md](/Users/axient/repository/praxis/specs/03-SPEC.md)
- [08-DISTRIBUTION.md](/Users/axient/repository/praxis/specs/08-DISTRIBUTION.md)

### 12.2 Task-Specific Context

Only include:

- the artifact kind being changed
- the specific packaging script
- platform details for the current release target

### 12.3 Context Reduction Rules

- one release artifact kind per prompt when possible
- keep UX and repository-contract docs out of narrow release-automation tasks

## 13. Logging, Status, and Observability

### 13.1 Operator-Visible Signals

- build result
- validation result
- published artifacts by channel

### 13.2 Logs and Traces

- release scripts should emit clear artifact paths and failure points
- published artifacts should remain traceable to one release version

## 14. Failure Model and Recovery Strategy

### 14.1 Failure Classes

- `package_failed`
- `artifact_invalid`
- `publish_failed`
- `channel_unavailable`

### 14.2 Recovery Behavior

- failed packaging blocks validation and publication
- failed validation blocks publication
- failed publication preserves the built artifact for retry or diagnosis

## 15. Safety, Boundaries, and Human Approval Policy

### 15.1 Always Do

- preserve source inspectability
- preserve contract consistency between CLI and desktop
- keep release artifact kinds explicit

### 15.2 Ask First

- adding a new release artifact kind
- introducing hosted dependencies into the core release path
- changing the primary release channel

### 15.3 Never Do

- never require a hosted control plane for core product value
- never ship a release that hides divergent CLI and desktop contracts
- never treat auto-update infrastructure as a hard requirement for v1

## 16. Reference Algorithms and Task Decomposition

### 16.1 Reference Algorithm

1. choose artifact kind
2. run the corresponding packaging script
3. validate the artifact
4. publish to GitHub Releases

### 16.2 Task Units and Parallelism

- source, CLI, and macOS packaging work may be validated independently
- publication remains sequential per release version

## 17. Validation, Commands, and Success Criteria

### 17.1 Commands

- `sh /Users/axient/repository/praxis/scripts/package-source.sh`
- `sh /Users/axient/repository/praxis/scripts/release-cli.sh`
- `sh /Users/axient/repository/praxis/scripts/release-macos.sh`

### 17.2 Validation Matrix

- source package remains auditable and reproducible
- CLI binary supports terminal and scriptable use
- macOS app supports visual inspection and management flows
- GitHub Releases is the primary publication channel

### 17.3 Acceptance and Conformance Gates

- all three release forms can be built and described coherently
- no mandatory hosted backend is required for core value
- CLI and desktop remain contract-consistent with the canonical spec

## 18. Implementation Checklist and Change Control

### 18.1 Required for Conformance

- source packaging path defined
- CLI packaging path defined
- macOS packaging path defined
- GitHub Releases channel defined

### 18.2 Recommended Extensions

- richer release validation automation
- notarization/signing enhancements where appropriate
- clearer release artifact metadata and changelog automation

### 18.3 Spec Update Triggers

- release artifact kinds change
- release scripts change materially
- release channel changes
- platform support boundary changes
