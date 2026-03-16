# 02. Blueprint

## 1. Final product shape

```text
External Sources
  -> Catalog Compiler
  -> Selection + Plan
  -> Reconciler
  -> Agent Runtime Targets
  -> Library + Agent Files + Benchmarks
```

Praxis has one center of gravity:

> Make external agent artifacts legible, selectable, exact to install, and exact to remove.

## 2. User-visible surfaces

### Primary surfaces

- **Discover** — inspect sources and begin install flows
- **Library** — manage everything already known, installed, or drafted
- **Create** — create or import new artifacts
- **Agent Files** — manage persistent instruction files by runtime and scope
- **Benchmarks** — compare candidate vs current artifacts

### Utility surfaces

- **Connections** — detected runtimes, paths, and source access
- **Health** — doctor, collisions, drift, stale ownership, invalid sources
- **Settings** — defaults, target profile preferences, creation defaults

## 3. Product loops

### 3.1 Discovery loop

```text
Paste source -> Inspect -> Choose -> Preview Plan -> Apply -> Land in Library
```

### 3.2 Agent-file loop

```text
Open Agent Files -> Choose runtime/slot -> Preview effective file -> Edit user block / select templates -> Apply
```

### 3.3 Creation loop

```text
Open Create -> Choose artifact type -> Select compatibility preset -> Fill metadata -> Preview output tree -> Save draft or write to repo
```

### 3.4 Promotion loop

```text
Draft or candidate artifact -> Benchmark against suite -> Review deltas -> Promote to Library/source repo
```

## 4. Product layers

### 4.1 Source layer

Accepts GitHub repositories and local directories.
External repos remain authoritative.

### 4.2 Catalog layer

Normalizes what the source contains into a single scanned representation:

- skills
- decks
- agent-file templates
- warnings
- recipe notes

### 4.3 Plan layer

Computes what would happen if the current selection were applied.
Plan is pure and non-mutating.

### 4.4 Reconcile layer

Applies deterministic copy-only installs, writes ownership, prunes stale managed artifacts, and composes agent files.

### 4.5 Runtime layer

Maps the selected artifacts into Codex, Claude Code, and Gemini CLI according to explicit target profiles.

### 4.6 Observation layer

Shows the result through Library, Agent Files, Health, and Benchmarks.

## 5. Final object system

- **Source** is where artifacts come from.
- **Catalog** is what the source contains.
- **Selection** is what the user wants.
- **Plan** is what would happen.
- **Manifest** is the desired state.
- **Lock** is the applied state.
- **Library** is the local management surface over installed state and local drafts.

## 6. What the desktop app is

The desktop app is not a marketplace UI.
It is a **consequence browser**.

Its job is to make these visible:

- what exists
- what is installed
- what will change
- what owns what
- what instruction files will be affected

## 7. What the CLI is

The CLI is not a fallback.
It is the authoritative operational interface.

The desktop app must map to CLI/core contracts instead of inventing separate semantics.

## 8. Final release shape

Praxis ships in three forms:

- source package
- CLI binary
- macOS desktop app

No server is required for the core product to function after installation.
