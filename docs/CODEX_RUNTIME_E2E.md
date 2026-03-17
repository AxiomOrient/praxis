# Codex Runtime E2E

This document defines the narrow live-provider verification path for Praxis.

## Goal

Re-verify that both of these paths work against a real `codex-runtime` provider, not the disabled fallback:

- `benchmark run --mode ai-judge`
- `create augment`

The live harness must prove three things:

1. the local machine can launch `codex-runtime` through the `codex` CLI
2. Praxis persists run/job state and evidence when a real provider is used
3. augment creates a new lineage-linked draft instead of only mutating queue state

## Preconditions

- `codex` CLI is installed on `PATH`
- `codex --version` is at least `0.104.0`
- local Codex authentication is already working
- operator accepts that the run will perform real provider calls

## Static Readiness Check

Run:

```bash
cargo run -q -p praxis-cli -- doctor --executor-provider codex-runtime
```

Expected checks:

- `codex-cli-ready`
- `codex-runtime-live-check-needed`

Optional model pin:

```bash
cargo run -q -p praxis-cli -- doctor --executor-provider codex-runtime --executor-model gpt-5-codex
```

## Live E2E Command

Run:

```bash
scripts/run_live_codex_runtime_e2e.sh
```

Optional model pin:

```bash
PRAXIS_LIVE_CODEX_RUNTIME_MODEL=gpt-5-codex scripts/run_live_codex_runtime_e2e.sh
```

The script performs:

1. static provider readiness via `praxis doctor`
2. live ignored integration test `live_codex_runtime`

## Live Test Contract

The ignored integration test creates an isolated temp repo and verifies:

- `ai-judge` finishes with `status = succeeded`
- benchmark evidence file exists and is non-empty
- `create augment` returns a new draft with `origin_kind = augment`
- augment lineage keeps `parent_version_id` and `augmentation_prompt`
- persisted jobs contain succeeded `benchmark-ai-judge` and `augment-draft`
- persisted jobs contain no failed entries after the live run

## Failure Interpretation

- `codex-cli-missing` or `codex-cli-version-too-old`
  The machine is not ready yet. Fix local CLI install first.
- `doctor` passes but the live test fails during prompt execution
  The machine likely has an auth/runtime issue. Validate local Codex login outside Praxis.
- benchmark succeeds but augment fails
  The shared executor is reachable, but draft-job or lineage application needs investigation.

## Why This Stays Opt-In

Live provider verification is intentionally not part of default CI:

- it depends on local operator auth
- it makes real provider calls
- it should fail loudly only when an operator explicitly asks for live verification
