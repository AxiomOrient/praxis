# praxis

**praxis** is a GitHub-first **management plane** for external agent artifacts and persistent agent files.

It is not a skill-pack repository.  
It discovers external repositories, scans for `SKILL.md` and runtime instruction files, and applies only the selected artifacts into Codex and Claude Code runtime targets with a **copy-only**, **lock-driven**, **no-stale-leftovers** model. Gemini-related work remains outside the current runtime-file contract until a separate adoption change is approved.

The canonical documentation set lives under `specs/` and is structured as a split, Symphony-style product specification package.

The canonical product target is:

- inspect external GitHub and local sources
- install / update / remove / sync selected skills and decks
- manage persistent agent files as contextual runtime workspace outputs
- create and adapt skills, decks, and agent-file templates
- benchmark and promote candidate artifacts

The final shell converges in phases:

- primary surfaces: Discover, Library, Create, Benchmarks
- utility surfaces: Connections, Health, Settings
- contextual flows: Plan preview, deck views, agent-file editor

## Product position

praxis is:

- a **source manager**
- a **selection planner**
- a **copy installer**
- a **state + lock reconciler**
- an **agent file manager**

praxis is not:

- a public registry
- a cloud service
- a symlink farm
- a skill authoring framework
- a monolithic “one blessed skills repo”

## The current shape

The best version of this product is still **CLI-authoritative** and **desktop-assisted**.

- The **CLI** is the system of record.
- The **desktop app** is a visual shell for inspect → plan → apply.
- The **manifest + lock** model remains the source of truth.
- The **source repo** remains external to praxis.

That keeps the product simple while still giving it a strong card-deck UX.

## Core principles

1. **GitHub is the source of truth.**
2. **One source install per scope.**
3. **Copy only. No symlinks.**
4. **Decks are optional; cards are first-class.**
5. **Agent files are composed deterministically, never clobbered blindly.**
6. **State is deterministic. Remove means remove.**
7. **Inspect → plan → apply is the primary interaction.**
8. **Codex and Claude Code are the only first-class runtime-file targets in the current contract.**

## Workspace layout

### Repo scope

```text
<repo>/.praxis/manifest.toml
<repo>/.praxis/lock.json
<repo>/.praxis/cache/
<repo>/.agents/skills/
<repo>/.claude/skills/
<repo>/AGENTS.md
<repo>/AGENTS.override.md
<repo>/AGENT.md          # optional compatibility alias
<repo>/CLAUDE.md
<repo>/.claude/CLAUDE.md
```

### User scope (macOS first)

```text
~/Library/Application Support/Praxis/manifest.toml
~/Library/Application Support/Praxis/lock.json
~/Library/Application Support/Praxis/cache/
~/.agents/skills/
~/.claude/skills/
~/.codex/AGENTS.md
~/.codex/AGENTS.override.md
~/.claude/CLAUDE.md
```

## Quick start

### CLI

```bash
cargo run -p praxis-cli -- --scope repo init
cargo run -p praxis-cli -- --scope repo inspect https://github.com/AxiomOrient/composable-skills
cargo run -p praxis-cli -- --scope repo plan https://github.com/AxiomOrient/composable-skills --agent codex --deck workflow --deck debug
cargo run -p praxis-cli -- --scope repo install https://github.com/AxiomOrient/composable-skills --agent codex --deck workflow --deck debug
cargo run -p praxis-cli -- --scope repo list
cargo run -p praxis-cli -- --scope repo sync
cargo run -p praxis-cli -- --scope repo doctor
```

### Desktop

```bash
cd apps/praxis-desktop
npm install
npm run tauri dev
```

## Distribution

The project includes local release scripts in `scripts/`, macOS app bundling configuration in `apps/praxis-desktop/src-tauri/tauri.conf.json`, and the canonical release boundary in `specs/08-DISTRIBUTION.md`.

The release posture is intentionally local-first:

- source package for auditability
- CLI archive for scripted users
- macOS app bundle for interactive users
- no GitHub Actions release automation
- no auto-update layer

## Status

This repository now treats `specs/` as the single canonical documentation package.
Legacy design drafts under `docs/` were removed to avoid conflicting product definitions.

Read in this order:

1. `specs/README.md`
2. `specs/00-HANDOFF.md`
3. `specs/01-PHILOSOPHY.md`
4. `specs/02-BLUEPRINT.md`
5. `specs/03-SPEC.md`
6. `specs/04-RUNTIME-TARGET-PROFILES.md`
7. `specs/05-REPOSITORY-CONTRACTS.md`
8. `specs/06-CREATION-SYSTEM.md`
9. `specs/07-UX-IA.md`
10. `specs/08-DISTRIBUTION.md`
11. `specs/99-REFERENCES.md`

### AI Agent 구성 하는 스킬을 기본 값으로 입력

- PM, BACKEND, FRONTEND, AI ANALYST, QA가 기본 개발팀
- 그 외의 마케팅 부터 다양한 업무 팀을(리서치 및 벤치마킹) 구성하는 예제는 다음 페이즈에 추가.
