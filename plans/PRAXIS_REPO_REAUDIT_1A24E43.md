# PRAXIS Repository Re-audit (commit gate: `1a24e43`)

## 0. Scope

This report is a **source-inspection audit** of the public `AxiomOrient/praxis` repository state reachable from `main` at analysis time.

What this report does:
- verifies that commit `1a24e43` is included
- analyzes the repository **file by file**
- separates **observed facts** from **design decisions**
- identifies contract drift between README / canonical specs / Rust core / desktop UI / examples
- proposes a tighter, more implementable project direction

What this report does **not** claim:
- it does not claim successful local build/test execution
- it does not infer runtime behavior beyond what is visible in repository source and docs
- it does not treat stale UI types or examples as authoritative over current core code

## 1. Gate result

**FACT** — The gate passes.

Observed basis:
1. `main` commit history publicly shows a single commit.
2. That single commit is `1a24e43`.
3. The direct commit page for `1a24e43` shows `0 parents`, meaning the current public branch state is rooted in that commit.

Therefore the current public repository **includes** commit `1a24e43`.

## 2. High-confidence current product reality

### 2.1 What is actually implemented now

**FACT** — The working core is a **deterministic GitHub/local source manager** with these capabilities:

- parse source input from GitHub URL, `owner/repo`, or local path
- scan sources for `SKILL.md`
- discover a limited set of agent-file templates from canonical filenames
- read optional `skills.deck.json`
- apply recipe-specific augmentation only for `garrytan/gstack`
- build a plan
- install/remove/sync via **copy-only** reconciliation
- maintain manifest (`.toml`) desired state + lock (`.json`) applied ownership state
- compose managed blocks into agent files while preserving user-authored content
- expose authoritative operations through CLI
- expose a desktop shell through Tauri

### 2.2 What is documented as the final target, but not implemented in the inspected code

**FACT** — The canonical spec package defines a larger product:
- `Library Plane`
- `Runtime Plane`
- `Evaluation Plane`
- local drafts / internal sources
- SQLite-backed library metadata
- benchmark storage and promotion evidence
- creation / import / fork / augment / promotion flows
- richer top-level UX surfaces

### 2.3 Current reality gap

**FACT** — The repo currently contains:
- an implemented **Runtime Plane core**
- a partially stale desktop shell
- a richer but not yet implemented spec package for the broader workspace product

The project is therefore **not yet** the full canonical spec product. It is a runtime-focused core with forward-looking specs.

## 3. Highest-signal mismatches

### 3.1 README vs canonical runtime-target spec

**FACT**
- `README.md` says Codex, Claude Code, and Gemini CLI are first-class targets.
- `specs/03-SPEC.md` and `specs/04-RUNTIME-TARGET-PROFILES.md` say first-class runtime-file install targets are **Codex and Claude**, while Gemini remains an integration target until a concrete runtime-file contract is adopted.
- current core code includes Gemini enums, paths, and slots, but default target selection in `manager.rs` still falls back to **Codex + Claude** when the request omits targets.

**Implication**
- runtime-target boundary is **not canonically aligned** across public docs and code.

### 3.2 Canonical terminology vs UI/examples

**FACT**
- `specs/00-HANDOFF.md` explicitly says **“Guides” is retired language** and the correct noun is **Agent Files**.
- current desktop code still uses `GuideKind`, `GuideAsset`, `guides`, `GuideEditor`, `GuidanceSnapshot`, `default_agents`, and `claude_project_guide_location`.
- the sample manifest in `examples/manifest.repo.sample.toml` still uses `guides` and `default_agents`.

**Implication**
- the terminology migration is incomplete and leaks into type contracts and examples.

### 3.3 Current backend vs desktop shell contract

**FACT**
- Tauri backend returns current core types such as `AgentFileSnapshot`.
- frontend API and types still expect legacy `GuidanceSnapshot`/`GuideState`-shaped data and old workspace settings fields.
- `App.svelte` still exposes top-level `plan` and `guides` tabs and reads fields that no longer exist in the current core model.

**Implication**
- desktop is currently behind the Rust core and should not be treated as the authoritative product contract.

### 3.4 Codebase cleanliness: dead legacy module

**FACT**
- `crates/praxis-core/src/guidance.rs` references removed legacy guide-centric types.
- `crates/praxis-core/src/lib.rs` does not export `guidance.rs`.

**Implication**
- `guidance.rs` is dead legacy source and should be archived or removed after explicit confirmation.

### 3.5 Spec package internal contradictions

**FACT**
- `specs/00-HANDOFF.md` says **Decks are not top-level navigation** and **Plan is not top-level navigation**.
- `specs/02-BLUEPRINT.md` primary surfaces are `Discover`, `Library`, `Create`, `Agent Files`, `Benchmarks`.
- `specs/07-UX-IA.md` says **My Skills and Decks** should be distinct top-level surfaces.
- `plans/PRAXIS_FINAL_SPEC.md` says guide/agent-file management should be a workspace output/editor rather than an independent top-level navigation item.

**Implication**
- the spec package itself still has unresolved navigation-layer drift.

### 3.6 Spec-writing system duplication

**FACT**
- the repo contains both `write-spec` and `spec-writing-standard`.
- the newer `spec-writing-standard` skill references `plans/PRAXIS_SPEC_WRITING_STANDARD.md`, but the committed standard actually lives at `docs/PRAXIS_SPEC_WRITING_STANDARD.md`.

**Implication**
- the toolchain that is supposed to generate canonical specs already contains a broken canonical reference path.

### 3.7 Workspace path completeness

**FACT**
- `workspace.rs` defines `gemini_skills_dir`.
- `ensure_workspace()` creates Codex/Claude skill roots, but not `gemini_skills_dir`.

**Implication**
- Gemini path handling in current code is incomplete even though the path model exists.

### 3.8 Spec portability issues

**FACT**
- multiple canonical spec documents contain absolute local file paths like `/Users/axient/repository/praxis/...`.

**Implication**
- the specs are less portable and less agent-friendly than they appear. This is unnecessary local-environment leakage.

## 4. File-by-file inventory

Legend:
- **ROOT** repository root/support file
- **SKILL** repository-local agent skill asset
- **APP** desktop implementation file
- **CODE** Rust implementation file
- **SPEC** canonical spec package file
- **DOC** other committed planning/analysis document
- **EXAMPLE** example/demo artifact
- **SCRIPT** release/packaging script
- **ASSET** packaging icon/bundle asset
- **STALE/DRIFT** file with observed contract drift relative to current core or canonical rules


### `.agents`
- `.agents/skills/spec-writing-standard/SKILL.md` — **STALE/DRIFT** — spec-writing skill. Newer AI-facing spec-writing skill. References missing path `plans/PRAXIS_SPEC_WRITING_STANDARD.md`; stale link.
- `.agents/skills/spec-writing-standard/agents/openai.yaml` — **SKILL** — skill metadata. OpenAI skill display metadata for `spec-writing-standard`.
- `.agents/skills/spec-writing-standard/assets/spec-writing-standard-template.md` — **SKILL** — spec template. Template scaffold for the newer spec-writing-standard skill.
- `.agents/skills/spec-writing-standard/scripts/check_spec_standard.py` — **SKILL** — validator. Validator for the newer spec-writing-standard skill.
- `.agents/skills/write-spec/SKILL.md` — **SKILL** — spec-writing skill. Older contract-style spec-writing skill with 18-section structure guidance.
- `.agents/skills/write-spec/agents/openai.yaml` — **SKILL** — skill metadata. OpenAI skill display metadata for `write-spec`.
- `.agents/skills/write-spec/assets/spec-template.md` — **SKILL** — spec template. Template scaffold for the older `write-spec` flow.
- `.agents/skills/write-spec/references/spec-writing-guide.md` — **SKILL** — reference guide. Explains the 18-section contract-style spec method used by `write-spec`.
- `.agents/skills/write-spec/scripts/check_spec.py` — **SKILL** — validator. Checks section continuity/coverage for `write-spec`.
- `.gitignore` — **ROOT** — repo hygiene. Ignore rules for build/dist/local artifacts.
- `Cargo.lock` — **ROOT** — dependency lockfile. Rust dependency lockfile.
- `Cargo.toml` — **ROOT** — workspace manifest. Rust workspace root; includes `praxis-core`, `praxis-cli`, and desktop Tauri crate; workspace version `1.1.0`.
- `README.md` — **ROOT** — public product summary. Current public definition: GitHub-first management plane, copy-only lock-driven install, CLI-authoritative, desktop-assisted. Also claims Codex/Claude/Gemini are first-class targets.

### `apps`
- `apps/praxis-desktop/index.html` — **APP** — frontend entry shell. Vite HTML entry for desktop webview.
- `apps/praxis-desktop/package-lock.json` — **APP** — dependency lockfile. NPM dependency lockfile.
- `apps/praxis-desktop/package.json` — **APP** — frontend manifest. Svelte 5 + Vite + Tauri 2 desktop package manifest, version `1.1.0`.
- `apps/praxis-desktop/src-tauri/Cargo.toml` — **APP** — desktop crate manifest. Tauri desktop Rust crate manifest.
- `apps/praxis-desktop/src-tauri/build.rs` — **APP** — build hook. Tauri build-script entry.
- `apps/praxis-desktop/src-tauri/capabilities/default.json` — **APP** — desktop capability config. Tauri capability/permission declaration.
- `apps/praxis-desktop/src-tauri/icons/128x128.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/128x128@2x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/32x32.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/64x64.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/Square107x107Logo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/Square142x142Logo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/Square150x150Logo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/Square284x284Logo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/Square30x30Logo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/Square310x310Logo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/Square44x44Logo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/Square71x71Logo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/Square89x89Logo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/StoreLogo.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-anydpi-v26/ic_launcher.xml` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-hdpi/ic_launcher.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-hdpi/ic_launcher_foreground.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-hdpi/ic_launcher_round.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-mdpi/ic_launcher.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-mdpi/ic_launcher_foreground.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-mdpi/ic_launcher_round.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-xhdpi/ic_launcher.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-xhdpi/ic_launcher_foreground.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-xhdpi/ic_launcher_round.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxhdpi/ic_launcher.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxhdpi/ic_launcher_foreground.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxhdpi/ic_launcher_round.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxxhdpi/ic_launcher.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxxhdpi/ic_launcher_foreground.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxxhdpi/ic_launcher_round.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/android/values/ic_launcher_background.xml` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/icon.icns` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/icon.ico` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/icon.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-20x20@1x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-20x20@2x-1.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-20x20@2x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-20x20@3x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-29x29@1x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-29x29@2x-1.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-29x29@2x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-29x29@3x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-40x40@1x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-40x40@2x-1.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-40x40@2x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-40x40@3x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-512@2x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-60x60@2x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-60x60@3x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-76x76@1x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-76x76@2x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-83.5x83.5@2x.png` — **ASSET** — packaging asset. Tauri-generated icon / platform bundle asset.
- `apps/praxis-desktop/src-tauri/src/lib.rs` — **APP** — desktop backend bridge. Tauri command bridge to current core functions; returns `AgentFileSnapshot`, not legacy `GuidanceSnapshot`.
- `apps/praxis-desktop/src-tauri/src/main.rs` — **APP** — desktop backend entry. Minimal Tauri main that calls `praxis_desktop_lib::run()`.
- `apps/praxis-desktop/src-tauri/tauri.conf.json` — **APP** — desktop bundle config. Tauri app/bundle/window configuration.
- `apps/praxis-desktop/src/App.svelte` — **STALE/DRIFT** — desktop shell. Primary desktop UI. Still models legacy top-level plan/guides surfaces and stale `default_agents`/`guides` contract.
- `apps/praxis-desktop/src/app.css` — **APP** — desktop styles. Global desktop styling.
- `apps/praxis-desktop/src/lib/api.ts` — **STALE/DRIFT** — desktop API client. Frontend Tauri invocation helpers. `guidance()` is typed as legacy `GuidanceSnapshot`; stale relative to backend.
- `apps/praxis-desktop/src/lib/components/Card.svelte` — **APP** — UI component. Generic card component.
- `apps/praxis-desktop/src/lib/components/DeckCard.svelte` — **APP** — UI component. Deck presentation card.
- `apps/praxis-desktop/src/lib/components/GuideEditor.svelte` — **APP** — UI component. Guide editor component; naming still uses retired `Guide` term.
- `apps/praxis-desktop/src/lib/components/InstalledSourceCard.svelte` — **APP** — UI component. Installed source summary card.
- `apps/praxis-desktop/src/lib/components/StarterSourceCard.svelte` — **APP** — UI component. Starter-source selection card.
- `apps/praxis-desktop/src/lib/i18n/en.ts` — **APP** — localization. English string catalog.
- `apps/praxis-desktop/src/lib/i18n/index.ts` — **APP** — localization. Locale router/helpers.
- `apps/praxis-desktop/src/lib/i18n/ja.ts` — **APP** — localization. Japanese string catalog.
- `apps/praxis-desktop/src/lib/i18n/ko.ts` — **APP** — localization. Korean string catalog.
- `apps/praxis-desktop/src/lib/starterSources.ts` — **APP** — desktop data. Preset starter-source catalog for the UI.
- `apps/praxis-desktop/src/lib/types.ts` — **STALE/DRIFT** — frontend contract types. Stale TS contract: still defines `GuideKind`, `guides`, `default_agents`, `claude_project_guide_location`, and old plan summary fields.
- `apps/praxis-desktop/src/main.ts` — **APP** — frontend bootstrap. Svelte mount entry.
- `apps/praxis-desktop/tsconfig.json` — **APP** — frontend config. TypeScript compiler configuration.
- `apps/praxis-desktop/vite.config.ts` — **APP** — frontend config. Vite config for Tauri dev/build.

### `crates`
- `crates/praxis-cli/Cargo.toml` — **CODE** — CLI crate manifest. Rust CLI manifest.
- `crates/praxis-cli/src/main.rs` — **CODE** — CLI surface. Authoritative operational interface: `init`, `inspect`, `plan`, `install`, `remove`, `list`, `sync`, `update`, `doctor`, `guidance show/set/paths`; supports Codex/Claude/Gemini enums and agent-file slots.
- `crates/praxis-core/Cargo.toml` — **CODE** — core crate manifest. Rust core manifest.
- `crates/praxis-core/src/agent_files.rs` — **CODE** — runtime composer. Deterministic managed-block composition for agent files; preserves user-authored content; writes Codex alias `AGENT.md` when enabled.
- `crates/praxis-core/src/guidance.rs` — **STALE/DRIFT** — legacy dead code. Old guide-based composer using removed types; not exported from `lib.rs`, so not part of current library build.
- `crates/praxis-core/src/lib.rs` — **CODE** — core module surface. Exports current build modules. Notably omits `guidance.rs`.
- `crates/praxis-core/src/manager.rs` — **CODE** — planner/reconciler. Implements inspect/plan/install/remove/sync/update/doctor. Default targets fall back to Codex+Claude only when none supplied.
- `crates/praxis-core/src/model.rs` — **CODE** — domain model. Current Rust data model for agents, slots, templates, manifest/lock, plan types, and placeholder library types.
- `crates/praxis-core/src/parser.rs` — **CODE** — input/parser. Parses GitHub/local source refs, canonicalizes source ids, parses `SKILL.md` frontmatter, validates skill naming.
- `crates/praxis-core/src/recipes.rs` — **CODE** — explicit augmentation. Single narrow recipe adapter for `garrytan/gstack`.
- `crates/praxis-core/src/source.rs` — **CODE** — source scanner. Fetches GitHub tarballs, caches immutable refs, safely unpacks archives, discovers skills/templates/decks, rejects symlink/hardlink tar entries.
- `crates/praxis-core/src/workspace.rs` — **CODE** — workspace IO. Resolves repo/user paths, creates workspace state, loads/saves manifest and lock, maps agent-file slots to concrete paths. Does not create `gemini_skills_dir` in `ensure_workspace()`.

### `docs`
- `docs/PRAXIS_REPOSITORY_ANALYSIS.md` — **DOC** — committed analysis artifact. A generated repository analysis document already committed into the repo; not the canonical spec package.
- `docs/PRAXIS_SPEC_WRITING_STANDARD.md` — **DOC** — committed standard. Working standard for AI-facing spec writing; layered artifact model and evidence rules.

### `examples`
- `examples/composable-skills.skills.deck.json` — **EXAMPLE** — example manifest. Standalone example deck manifest.
- `examples/manifest.repo.sample.toml` — **STALE/DRIFT** — example manifest. Sample workspace manifest using stale fields (`default_agents`, `guides`, `claude_project_guide_location`).
- `examples/sources/demo-cards/AGENTS.md` — **EXAMPLE** — example template. Demo Codex/shared repository guidance block.
- `examples/sources/demo-cards/CLAUDE.md` — **EXAMPLE** — example template. Demo Claude guidance block.
- `examples/sources/demo-cards/debug-root-cause/SKILL.md` — **EXAMPLE** — example skill. Debugging skill using reproduce→observe→hypothesize→verify→RCA flow.
- `examples/sources/demo-cards/plan-cleanly/SKILL.md` — **EXAMPLE** — example skill. Planning skill emphasizing assumptions/constraints/acceptance checks.
- `examples/sources/demo-cards/ship-checklist/SKILL.md` — **EXAMPLE** — example skill. Pre-ship checklist skill.
- `examples/sources/demo-cards/skills.deck.json` — **EXAMPLE** — example manifest. Demo deck manifest with `core` and `workflow`.
- `examples/sources/demo-cards/workflow-build-release/SKILL.md` — **EXAMPLE** — example skill. Higher-level workflow demo skill.

### `plans`
- `plans/PRAXIS_FINAL_SPEC.md` — **DOC** — committed plan/spec. Repository-local refinement doc that preserves current runtime core and defines final product as Library Plane + Runtime Plane + Evaluation Plane.

### `scripts`
- `scripts/package-source.sh` — **SCRIPT** — release script. Builds source ZIP while excluding dist/target/node_modules/.git.
- `scripts/release-cli.sh` — **SCRIPT** — release script. Builds release CLI binary for a target triple and archives it.
- `scripts/release-macos.sh` — **SCRIPT** — release script. Builds desktop app with Vite+Tauri and zips the resulting `.app`.

### `specs`
- `specs/00-HANDOFF.md` — **SPEC** — canonical spec package. Compressed handoff. Declares `Guides` retired, Decks not top-level, Plan not top-level.
- `specs/01-PHILOSOPHY.md` — **SPEC** — canonical spec package. Product thesis and non-negotiable principles: repository-owned truth, copy-only, management-plane posture.
- `specs/02-BLUEPRINT.md` — **SPEC** — canonical spec package. High-level final product shape and primary/utility surfaces.
- `specs/03-SPEC.md` — **SPEC** — canonical spec package. Canonical product contract: local-first workspace with Library Plane, Runtime Plane, Evaluation Plane.
- `specs/04-RUNTIME-TARGET-PROFILES.md` — **SPEC** — canonical spec package. Runtime mapping contract. Codex/Claude are first-class runtime-file targets; Gemini is integration-only until documented otherwise.
- `specs/05-REPOSITORY-CONTRACTS.md` — **SPEC** — canonical spec package. Repository/source contract: optional manifests enrich UX but portable sources remain useful; arbitrary Markdown must not be inferred as templates.
- `specs/06-CREATION-SYSTEM.md` — **SPEC** — canonical spec package. Creation/import/draft/promotion contract.
- `specs/07-UX-IA.md` — **STALE/DRIFT** — canonical spec package. UX/IA contract. Currently conflicts with `00-HANDOFF.md` and `02-BLUEPRINT.md` on top-level surfaces.
- `specs/08-DISTRIBUTION.md` — **SPEC** — canonical spec package. Release shape/distribution contract. Contains absolute local file paths in examples/commands.
- `specs/99-REFERENCES.md` — **SPEC** — canonical spec package. External references, including OpenAI Symphony.
- `specs/README.md` — **SPEC** — canonical spec package. Spec package index; says split package is intentional and `03-SPEC.md` is canonical.

## 5. Fact-based direction correction

### 5.1 Observed stable core

**FACT** — The strongest, already-working center of gravity is:

> deterministic source resolution → inspect → selection → plan → copy-only apply/remove/sync → ownership lock → agent-file composition

This is the product center that already exists in code.

### 5.2 Observed future target

**FACT** — The strongest forward product definition in canonical specs is:

> local-first workspace = Library Plane + Runtime Plane + Evaluation Plane

This is the clearest broadening direction already written in specs.

### 5.3 Decision: the project should not pivot away from the current core

**DECISION**
The correct evolution is **not** “replace the current product with a different one.”

It should be:

> **Preserve the current installer/reconciler as the Runtime Plane, then build the missing Library Plane and Evaluation Plane around it.**

Reason:
- it matches current code reality
- it matches `specs/03-SPEC.md`
- it avoids discarding the only already-concrete, deterministic subsystem

## 6. Final-form refinement

### 6.1 Product definition

**DECISION**
Praxis final form should be fixed as:

> a **local-first skill workspace** that ingests external sources into a managed local library, resolves selected artifacts into deterministic runtime outputs, and stores evaluation evidence used for promotion.

### 6.2 Planes

**DECISION**
1. **Runtime Plane** — current Rust core.  
   Responsibilities:
   - inspect
   - plan
   - copy-only apply/remove/sync
   - manifest/lock ownership
   - agent-file slot composition
   - collision/drift reporting

2. **Library Plane** — new persistent local artifact layer.  
   Responsibilities:
   - normalized local versions of imported/source artifacts
   - provenance
   - drafts
   - internal collections
   - metadata queries
   - source refresh bookkeeping

3. **Evaluation Plane** — new evidence layer.  
   Responsibilities:
   - benchmark suites
   - candidate vs current comparisons
   - persisted results
   - promotion evidence

### 6.3 Runtime target boundary

**DECISION**
Until a concrete Gemini runtime-file contract is formally adopted, the product boundary should be:

- first-class runtime-file install targets: **Codex, Claude**
- explicit integration target only: **Gemini**

Why this decision:
- it matches `specs/03-SPEC.md`
- it matches `specs/04-RUNTIME-TARGET-PROFILES.md`
- it matches current default-target behavior in `manager.rs`
- it removes the README/code/spec contradiction

### 6.4 Navigation and surface model

**DECISION**
Use this navigation model:

Primary:
- Discover
- Library
- Create
- Benchmarks

Utility:
- Connections
- Health
- Settings

Workspace-scoped detail surfaces:
- Agent Files
- Deck views
- Plan preview

Why:
- `00-HANDOFF.md` explicitly rejects Plan as top-level
- `00-HANDOFF.md` explicitly rejects Decks as top-level
- current code already supports plan as a flow state rather than a domain center
- Agent Files are operational outputs tied to workspace/runtime context

This decision intentionally resolves the `00/02/07/plans` drift instead of preserving it.

### 6.5 Authoritative interfaces

**DECISION**
- **CLI/core** remains the authoritative product contract during convergence.
- **Desktop** is a visual shell and must be brought into contract parity before new surfaces are expanded.

### 6.6 Storage model

**DECISION**
Retain:
- manifest TOML = desired workspace state
- lock JSON = applied ownership state

Add:
- SQLite = library metadata / provenance / jobs / benchmark records
- filesystem artifact store = immutable artifact contents and draft contents

### 6.7 Repository contract boundary

**DECISION**
Keep repository compatibility layers strict:
- portable open-standard discovery is baseline
- optional manifests enrich UX
- recipes remain narrow and explicit
- arbitrary Markdown must never be inferred as an agent-file template without declaration or recipe support

### 6.8 Creation boundary

**DECISION**
Creation should be added only as a contract-preserving system:
- create
- import
- fork
- rename
- augment
- preview
- promote

Not as a generic note editor or freeform IDE surface.

## 7. Concrete improvement order

### Phase 0 — contract convergence
1. Remove or archive dead `guidance.rs`.
2. Replace legacy desktop `Guide*` types with current `AgentFile*` contracts.
3. Remove `default_agents`, `guides`, and `claude_project_guide_location` from frontend/example contracts.
4. Fix broken spec-writing-standard canonical path reference.
5. Remove absolute local file paths from spec docs.
6. Make README runtime-target statement match canonical runtime-target boundary.

### Phase 1 — Runtime Plane hardening
1. Ensure `gemini_skills_dir` creation or explicitly drop Gemini runtime-root handling from current code until promoted.
2. Add contract tests for:
   - source parsing
   - deck resolution
   - install planning
   - conflict refusal
   - managed-block agent-file composition
   - manifest/lock rewrite invariants
3. Add desktop/backend schema conformance tests.

### Phase 2 — Library Plane
1. Add SQLite metadata store.
2. Add artifact store layout.
3. Add provenance model.
4. Add internal sources:
   - `internal:drafts`
   - `internal:collections`
5. Convert source refresh into library snapshot ingestion.

### Phase 3 — Evaluation Plane
1. Benchmark suite definitions.
2. Candidate/current artifact comparison records.
3. Promotion evidence storage.
4. Benchmark surfaces in UI/CLI.

### Phase 4 — Create system
1. Draft store
2. Compatibility presets
3. Deterministic preview tree
4. Promotion writer
5. Lineage tracking

### Phase 5 — UI restructuring
1. Converge desktop on current core contracts.
2. Replace current stale tabs with final navigation model.
3. Surface Decks/Agent Files as contextual library/runtime subviews rather than top-level drift surfaces.

## 8. Non-negotiable cleanup rules

**DECISION**
1. README, examples, desktop types, and specs may not define new contracts independently of core + canonical spec.
2. Any terminology migration must be global. `Guide` and `Agent File` must not coexist as peer nouns.
3. A runtime is not “first-class” unless:
   - slot/root contract is documented
   - planner supports it
   - workspace creation supports it
   - examples and UI reflect it
4. A spec document is not canonical if it contradicts `03-SPEC.md` and does not explicitly declare itself as a proposal or divergence log.
5. Broken spec-tool references must be treated as product bugs, not doc nits.

## 9. Short conclusion

**FACT**
The repository already contains a strong deterministic Runtime Plane.

**DECISION**
The best path is to turn that Runtime Plane into one of three explicit planes, then build Library and Evaluation around it, while first eliminating contract drift in desktop/UI/examples/spec tooling.

That path is both:
- the most faithful to the current code
- the most faithful to the strongest canonical specs
- the least wasteful migration path
