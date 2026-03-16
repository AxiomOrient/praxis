# [System Name] Specification

Status: Draft v1

Purpose: Define [system, workflow, or feature] as an AI-facing implementation contract.

## 1. Problem Statement

[What the system is, who it serves, what problem it solves, and the important boundary.]

## 2. Goals and Non-Goals

### 2.1 Goals

- [Goal]

### 2.2 Non-Goals

- [Non-goal]

## 3. System Overview

### 3.1 Main Components

1. `[Component]`
   - [Responsibility]

### 3.2 Abstraction Levels

1. `[Layer]`
   - [Meaning]

### 3.3 External Dependencies

- [Dependency]

### 3.4 Project Structure and Key Paths

- `[path]` — [why it matters]

## 4. Core Domain Model

### 4.1 Entities

#### 4.1.1 [Entity]

Definition:

[What it is.]

Fields:

- `[field]`

Lifecycle:

[State or lifecycle note.]

### 4.2 Stable Identifiers and Normalization Rules

- `[identifier rule]`

## 5. Domain Contract

### 5.1 [Contract Name]

- [Rule]

## 6. Configuration and Input Contract

### 6.1 Precedence and Defaults

- [Rule]

### 6.2 Validation and Coercion

- [Rule]

## 7. Lifecycle or State Model

### 7.1 States

1. `[State]`
   - [Meaning]

### 7.2 Transitions and Guards

- `[Trigger or guard]`

## 8. Primary Workflows and Reconciliation

### 8.1 Primary Flow

1. [Step]

### 8.2 Failure or Retry Branches

- [Branch rule]

## 9. Storage, Ownership, and Safety Boundaries

### 9.1 Managed Locations

- `[path]` — [ownership rule]

### 9.2 Destructive Boundaries

- [Rule]

## 10. Execution or Interface Contract

### 10.1 Invocation Contract

- [Rule]

## 11. External Integration Contract

### 11.1 Required Operations

- [Rule]

## 12. Context Packaging and Prompt Inputs

### 12.1 Required Context

- [Always include]

### 12.2 Task-Specific Context

- [Only include when relevant]

### 12.3 Context Reduction Rules

- [How to split or summarize]

## 13. Logging, Status, and Observability

### 13.1 Operator-Visible Signals

- [Signal]

### 13.2 Logs and Traces

- [Rule]

## 14. Failure Model and Recovery Strategy

### 14.1 Failure Classes

- `[failure_class]`

### 14.2 Recovery Behavior

- [Recovery rule]

## 15. Safety, Boundaries, and Human Approval Policy

### 15.1 Always Do

- [Rule]

### 15.2 Ask First

- [Rule]

### 15.3 Never Do

- [Rule]

## 16. Reference Algorithms and Task Decomposition

### 16.1 Reference Algorithm

1. [Step]

### 16.2 Task Units and Parallelism

- [Decomposition rule]

## 17. Validation, Commands, and Success Criteria

### 17.1 Commands

- `command` — [purpose]

### 17.2 Validation Matrix

- [Area] — [Expected evidence]

### 17.3 Acceptance and Conformance Gates

- [Gate]

## 18. Implementation Checklist and Change Control

### 18.1 Required for Conformance

- [Required item]

### 18.2 Recommended Extensions

- [Recommended item]

### 18.3 Spec Update Triggers

- [When the spec must be revised]
