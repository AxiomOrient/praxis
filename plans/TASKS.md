# Praxis Execution Ledger

## Phase Roadmap

| Phase | Objective | Exit Gate |
|------|------|------|
| P0 | Canonical convergence | plans/specs/README/examples/desktop agree on IA and runtime boundary |
| P1 | Desktop shell convergence | desktop nav matches final surface model and still builds |
| P2 | Runtime Plane hardening | runtime conformance tests cover planner/reconciler invariants |
| P3 | Library Plane | SQLite metadata + artifact store + provenance exist |
| P4 | Evaluation Plane | benchmark evidence is stored and surfaced |
| P5 | Create system | draft/preset/preview/promotion flow works against repository contracts |
| P6 | External execution and cooperative jobs | optional AI execution uses persisted jobs with lease/reclaim semantics |
| P7 | Lineage UX and promotion review | fork/augment ancestry and promotion evidence are visible in one review flow |

## Active Task Rows

| ID | Status | Goal | Scope | Done When | Evidence | Depends On |
|----|--------|------|-------|-----------|----------|------------|
| P0A-01 | done | Re-anchor the canonical IA contract in planning artifacts | `plans/IMPLEMENTATION-PLAN.md`, `plans/PRAXIS_FINAL_FORM_REFINEMENT.md`, `plans/PRAXIS_FINAL_SPEC.md` | The phase roadmap and IA authority are explicit and no longer defer IA redesign to a later phase | Rewrote `plans/IMPLEMENTATION-PLAN.md` and `plans/TASKS.md` around the full P0→P5 roadmap and updated `plans/PRAXIS_FINAL_SPEC.md` to use `Library` top-level plus contextual agent-file flows | - |
| P0A-02 | done | Converge canonical specs onto one IA story | `specs/00-HANDOFF.md`, `specs/02-BLUEPRINT.md`, `specs/07-UX-IA.md` | Specs agree that `Discover/Library/Create/Benchmarks` are primary, `Connections/Health/Settings` are utility, and `Plan/Decks/Agent Files` are contextual | Updated all three specs to the same surface model; grep now shows one consistent IA direction instead of competing top-level contracts | P0A-01 |
| P0A-03 | done | Align README to the same phased product and IA boundary | `README.md` | README explains the phased final form and does not advertise stale shell structure as canonical | README now states primary/utility/contextual surface groups and treats agent files as contextual runtime outputs | P0A-02 |
| P1A-01 | done | Reshape desktop top-level nav around the new IA | `apps/praxis-desktop/src/App.svelte`, `apps/praxis-desktop/src/lib/i18n/*.ts` | Top-level nav removes `Plan` and `Guides`, introduces final-surface-oriented destinations, and keeps plan/agent files contextual | `App.svelte` nav now exposes `Discover`, `Library`, `Create`, `Benchmarks`, `Connections`, `Health`, `Settings`; no top-level nav button points at `plan` or `agent-files` | P0A-02 |
| P1A-02 | done | Re-home plan preview and agent-file editing into contextual flows | `apps/praxis-desktop/src/App.svelte`, `apps/praxis-desktop/src/lib/components/AgentFileEditor.svelte` | Plan preview is entered from Discover/Library flow and agent-file editing is reached contextually from Library/workspace, not top-level nav | `plan` remains a preview state entered from Discover actions; agent-file editor is opened from Library advanced actions; desktop build passed after the change | P1A-01 |
| P1A-03 | done | Add bounded placeholders for future primary/utility surfaces that are not implemented yet | `apps/praxis-desktop/src/App.svelte`, `apps/praxis-desktop/src/lib/i18n/*.ts` | Create, Benchmarks, and Connections exist as explicit bounded placeholders so the shell matches the roadmap without pretending functionality exists | Added shell surfaces and localized placeholder copy for Create, Benchmarks, and Connections; build passed | P1A-01 |
| P1A-04 | done | Verify the current convergence slice | repo | Rust tests pass, desktop builds, and IA grep gates are clean | `cargo test --workspace` passed, `npm run build` passed in `apps/praxis-desktop`, and grep checks confirmed no top-level `Plan`/`Guides` nav plus one consistent IA direction across active docs | P0A-03, P1A-02, P1A-03 |

## Active Runtime Hardening Tasks

| ID | Status | Goal | Scope | Done When | Evidence | Depends On |
|----|--------|------|-------|-----------|----------|------------|
| P2A-01 | done | Fix runtime workspace path invariants | `crates/praxis-core/src/workspace.rs` | Workspace initialization creates every declared managed skill root needed by the current path model | `ensure_workspace()` now creates `gemini_skills_dir`, and a new unit test proves Codex/Claude/Gemini skill roots plus manifest/lock files are created | P1A-04 |
| P2A-02 | done | Make planner defaults and validation target-profile aware | `crates/praxis-core/src/manager.rs`, `crates/praxis-core/src/model.rs` | Empty target selections resolve from `target_profile`, and unsupported Gemini runtime-file targets/profiles are rejected explicitly instead of being silently accepted | Added `TargetProfile` helpers, made `build_install_record()`/`normalize_targets()` profile-aware, and wired doctor validation to report unsupported Gemini runtime-file targets instead of silently accepting them | P2A-01 |
| P2A-03 | done | Add runtime conformance regression tests | `crates/praxis-core/src/manager.rs`, `crates/praxis-core/src/workspace.rs` | Runtime hardening behavior is protected by tests for path creation, profile defaults, and validation failures | Added tests for workspace root creation, target-profile-derived defaults, explicit Gemini rejection, and Gemini-profile rejection | P2A-02 |
| P2A-04 | done | Add desktop/backend schema conformance checks | `apps/praxis-desktop/src-tauri/src/lib.rs` | Backend serialization tests lock the current field names used by the desktop contract | Added Tauri serialization tests that assert current workspace/install/agent-file JSON keys such as `target_profile`, `agent_file_templates`, `agent_file_actions`, `total_agent_file_actions`, and `slot` | P2A-02 |
| P2A-05 | done | Verify the runtime hardening slice | repo | Rust tests and desktop build pass after the runtime hardening changes | `cargo test --workspace` passed with 17 total unit tests green, and `npm run build` passed in `apps/praxis-desktop` after the runtime hardening changes | P2A-03, P2A-04 |

## Planned Later-Phase Tasks

| ID | Status | Goal | Scope | Done When | Evidence | Depends On |
|----|--------|------|-------|-----------|----------|------------|
| P3-01 | done | Library metadata and artifact store | new library modules + storage schema | SQLite and artifact store back the Library Plane | Added `library` storage authority with SQLite schema, filesystem artifact store/import manifests, library stats in `WorkspaceSnapshot`, and regression tests covering DB bootstrap plus source snapshot import | P2A-05 |
| P4-01 | done | Benchmark/evaluation persistence | benchmark modules + CLI/UI surfaces | benchmark runs and promotion evidence persist | Added SQLite-backed benchmark suites/runs, deterministic `praxis benchmark run` persistence, workspace evaluation summaries, desktop Benchmarks surface wiring, and regression tests for suite bootstrap plus run persistence | P3-01 |
| P5-01 | done | Create system and promotion writer | create flows + preview/export path | draft/preset/preview/promotion flows work | Added draft storage in the local artifact store, CLI create/preview/promote flows, repo-contract promotion writer to `.agents/skills/<name>`, desktop Create draft summaries, and regression tests for draft preview/promotion | P4-01 |
| P6-01 | done | Add external LLM executor boundary | provider adapter contract in core/spec/CLI | Praxis has one optional external executor contract with `codex-runtime` as the supported first adapter and no product logic depends on provider-specific session state | Added `executor.rs`, `ExternalExecutorConfig`, CLI executor flags, `codex-runtime` integration, and regression tests for enabled/disabled executor config behavior | P5-01 |
| P6-02 | done | Implement persisted cooperative job leasing | job storage + worker loop + CLI/UI summaries | Long-running benchmark and AI review work runs through persisted jobs with `queued`, `leased`, `running`, `succeeded`, `failed`, `cancelled` plus reclaimable stale leases | Added `jobs.rs`, SQLite-backed jobs table, lease/reclaim/cancel/retry flow, `praxis jobs work`, desktop job queue surface, and tests for stale lease reclaim plus retry after failure | P6-01 |
| P6-03 | done | Route human review and AI judge through the same job path | evaluation/create orchestration + desktop review queue | Human review and AI judge are durable resumable jobs instead of bespoke flows | `ai-judge` benchmark mode now queues worker jobs with evidence paths, human review submission runs through the same job system, draft augment jobs persist logs, and desktop shows queue status plus retry/cancel controls | P6-02 |
| P7-01 | done | Add minimal lineage history surfaces | Library/Create desktop views + core summaries | Users can see parent version, origin kind, and creation path for every draft and promoted artifact without opening raw metadata | Draft records now persist lineage metadata for create/fork/augment/promotion, desktop Create lists origin + parent summary, and Tauri serialization tests lock lineage payload keys | P5-01 |
| P7-02 | done | Add focused promotion review UX | Benchmarks/Create review surfaces | Promotion review shows ancestry, changed files, benchmark evidence, and review state in one place | Draft preview now includes promotion review summary, Create shows lineage + review stats in one panel, Benchmarks exposes persisted run evidence with jobs, and desktop build/tests passed on the converged surface | P6-03, P7-01 |
