#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

export PRAXIS_LIVE_CODEX_RUNTIME="${PRAXIS_LIVE_CODEX_RUNTIME:-1}"

doctor_args=()
if [[ -n "${PRAXIS_LIVE_CODEX_RUNTIME_MODEL:-}" ]]; then
  doctor_args+=(--executor-model "$PRAXIS_LIVE_CODEX_RUNTIME_MODEL")
fi

if (( ${#doctor_args[@]} > 0 )); then
  cargo run -q -p praxis-cli -- doctor --executor-provider codex-runtime "${doctor_args[@]}"
else
  cargo run -q -p praxis-cli -- doctor --executor-provider codex-runtime
fi
cargo test -p praxis-core --test live_codex_runtime -- --ignored --nocapture
