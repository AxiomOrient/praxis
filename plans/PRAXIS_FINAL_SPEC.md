# PRAXIS Final Canonical Specification

Status: Draft v1  
Purpose: 현재 구현된 source manager 코어를 보존하면서, renewal 목표를 흡수한 **로컬 우선 skill workspace**의 최종 구현 기준을 정의한다.  
Reference style: OpenAI `symphony` SPEC의 구성 방식(문제 정의 → 목표/비목표 → 시스템 개요 → 도메인 모델 → 계약 → 검증/오류 표면)을 따른다.

---

## 0. 규범 구분

이 문서에는 두 종류의 규범이 있다.

- **Inherited invariant**  
  현재 코드에서 이미 작동 중이며 유지해야 하는 규칙.
- **Normative final decision**  
  최종 제품 구현을 위해 이 문서가 새로 확정하는 규칙.

### 0.1 Inherited invariants

다음은 현행 코드/문서에서 확인되며 최종 제품에서도 보존해야 한다.

1. install은 **copy-only**다.
2. apply는 **plan → validate → apply → prune → lock write** 순서다.
3. stale artifact 정리는 **lock ownership**에 의해 결정된다.
4. unmanaged 파일/디렉터리를 조용히 덮어쓰지 않는다.
5. guide 관리 시 사용자 작성 내용은 보존하고 praxis managed block만 갱신한다.
6. CLI는 자동화 기준 구현이며 desktop은 보조 UX다.
7. 제품은 로컬 우선이며 필수 클라우드 백엔드를 요구하지 않는다.
8. source intake와 file copy 과정에서 symlink/hardlink를 신뢰하지 않는다.

### 0.2 Final decisions

이 문서는 다음을 확정한다.

1. praxis의 최종형태는 **Library Plane + Runtime Plane + Evaluation Plane**으로 구성된 local-first skill workspace다.
2. 현재 `praxis-core` installer/reconciler는 **Runtime Plane**으로 보존한다.
3. library metadata의 authoritative store는 **SQLite**다.
4. skill/deck/guide 실제 파일 내용의 authoritative store는 **filesystem artifact store**다.
5. background job은 필요하지만 **resident daemon을 요구하지 않는다**. 대신 persisted cooperative job worker를 사용한다.
6. GitHub/local source는 library의 origin이며, runtime apply는 library-resolved snapshot/version을 사용한다.
7. guide는 독립 top-level navigation이 아니라 **workspace context의 output/editor**가 된다.

---

## 1. Problem Statement

현재 praxis는 외부 GitHub/local source를 스캔하고, 선택한 skill/deck/guide를 Codex/Claude 대상 디렉터리에 결정적으로 설치하는 source manager다.  
이 모델은 runtime install에는 충분하지만, 다음 요구를 직접 해결하지 못한다.

1. source snapshot을 넘는 **local library/version history**
2. imported skill과 user-created draft의 **동일한 작업 공간 내 공존**
3. fork / rename / augment / promote 같은 **authoring lifecycle**
4. benchmark와 evaluation을 통한 **품질 판정**
5. source, library, runtime, benchmark를 한 시스템 안에서 잇는 **일관된 provenance**

최종 praxis는 이 공백을 채워야 한다. 단, 현재 작동 중인 deterministic runtime installer를 희생해서는 안 된다.

---

## 2. Goals and Non-Goals

### 2.1 Goals

1. GitHub/local/internal source로부터 skill/deck/guide를 가져와 versioned local library에 저장한다.
2. draft/fork/augment/create를 통해 skill을 로컬에서 편집하고 재버전한다.
3. deck을 명시적 컬렉션과 source-derived 컬렉션 모두로 지원한다.
4. benchmark suite/run을 통해 후보 skill version의 품질을 평가한다.
5. workspace 단위로 plan/apply/remove/sync/doctor를 계속 지원한다.
6. Codex/Claude runtime install은 현재와 같은 deterministic/lock-driven 모델을 유지한다.
7. desktop과 CLI는 동일한 core contract 위에 올라간다.
8. 재시작 후에도 library/jobs/benchmark history를 복구할 수 있다.
9. 필수 SaaS 없이 동작한다.

### 2.2 Non-Goals

1. hosted cloud registry
2. multi-tenant team backend
3. social marketplace / publishing network
4. distributed scheduler / remote worker farm
5. auto-update service
6. agent runtime 자체(Codex/Claude/Gemini)의 구현 대체
7. arbitrary binary plugin execution framework
8. Git hosting provider 전반의 완전한 abstraction  
   - v1-final의 primary remote source는 GitHub다.
9. source repository의 전체 development workflow 대행  
   - praxis는 skill workspace이지 general Git client가 아니다.

---

## 3. Product Boundary

### 3.1 Praxis owns

1. source intake (GitHub/local/internal)
2. source scan 및 source-to-library mapping
3. local library metadata와 artifact storage
4. skill create / fork / rename / augment
5. deck composition / synthesis / personal collection
6. workspace plan / apply / remove / sync / doctor
7. guide rendering / merge / write
8. benchmark suite / run / promotion recommendation
9. toolchain connection detection / health checks
10. persisted cooperative jobs
11. CLI and desktop interfaces

### 3.2 Praxis does not own

1. remote registry hosting
2. remote collaboration backend
3. GitHub repository management beyond source fetch/export convenience
4. cloud secrets management
5. enterprise policy server
6. general-purpose workflow orchestration outside skill/library/runtime/evaluation scope

---

## 4. System Overview

### 4.1 Main Components

1. **Source Adapter Layer**
   - GitHub source canonicalization
   - local path canonicalization
   - source fetch/cache
   - safe unpack and scan

2. **Normalization Layer**
   - converts source-native files into canonical `SkillVersion`, `DeckVersion`, `GuideVersion`, `RecipeBundleVersion`

3. **Library Store**
   - SQLite metadata DB
   - artifact directory store
   - provenance relationships
   - search/filter/sort state

4. **Workspace Runtime Layer**
   - plan generation
   - collision detection
   - unmanaged conflict detection
   - copy-only apply
   - deterministic prune
   - lock write
   - guide merge

5. **Creator Layer**
   - draft creation
   - fork/rename/augment
   - authoring validation
   - export/import

6. **Benchmark Layer**
   - suite management
   - run scheduling
   - AI judge / human A/B / hybrid evaluation
   - promotion recommendation

7. **Connections & Health Layer**
   - Codex/Claude/Gemini executable/auth detection
   - runtime path health
   - workspace doctor views

8. **Job Engine**
   - persisted jobs
   - cooperative worker lease
   - retry/backoff
   - progress/log tracking

9. **CLI Layer**
   - authoritative automation interface

10. **Desktop Layer**
    - interactive workspace UI
    - plan review
    - editor
    - benchmark review
    - health/connection surfaces

### 4.2 Abstraction Levels

1. **Source**  
   외부/내부 origin. GitHub repo, local path, internal drafts group.

2. **Snapshot**  
   특정 시점의 source content capture.

3. **Library Version**  
   snapshot 또는 local draft로부터 정규화된 immutable version record.

4. **Workspace Install Selection**  
   특정 workspace가 원하는 source/deck/skill/guide 선택.

5. **Workspace Lock**  
   실제로 어떤 version이 어떤 target path를 소유하는지 기록한 resolved state.

6. **Benchmark Result**  
   특정 version candidate에 대한 품질 평가 결과.

---

## 5. Storage and Filesystem Contract

## 5.1 State roots

### User state root

`state_root = platform_app_data_dir("Praxis")`

macOS 구현 기준 예시는 다음과 같다.

```text
~/Library/Application Support/Praxis
```

### Repo workspace root

repo scope workspace state root:

```text
<repo>/.praxis
```

## 5.2 Authoritative stores

Praxis는 두 개의 authoritative store를 가진다.

1. **Library metadata store** — SQLite
2. **Workspace desired/resolved state store** — manifest TOML + lock JSON

이 분리는 필수다.

- library는 source history / versions / benchmark / jobs / connections를 저장한다.
- workspace manifest/lock은 각 runtime target의 desired/resolved apply 상태를 저장한다.

## 5.3 User state directory layout

```text
<state_root>/
  db/
    praxis.db
  library/
    skills/<skill_id>/<skill_version_id>/
    guides/<guide_version_id>.md
    decks/<deck_version_id>.json
    bundles/<bundle_version_id>/
    imports/<source_snapshot_id>.json
  cache/
    sources/<source_cache_key>.tar.gz
    unpack/<source_cache_key>/
  jobs/
    <job_id>.json
  logs/
    jobs/<job_id>.log
  exports/
  temp/
```

## 5.4 Workspace directory layout

### Repo scope

```text
<repo>/
  .praxis/
    manifest.toml
    lock.json
  .agents/skills/
  .claude/skills/
  AGENTS.md
  AGENTS.override.md
  AGENT.md           # optional alias
  CLAUDE.md
  .claude/CLAUDE.md
```

### User scope

Inherited runtime target locations:

```text
~/.agents/skills/
~/.claude/skills/
CODEX_HOME or ~/.codex        # codex guide targets
~/Library/Application Support/Praxis   # state root
```

Claude guide target path는 setting `claude_guide_location`에 따라 결정한다.

## 5.5 SQLite requirement

Normative final decision:

- library metadata store는 SQLite여야 한다.
- 이유:
  1. source snapshot ↔ imported item ↔ version ↔ benchmark ↔ install 관계가 교차 조회를 요구한다.
  2. restart recovery와 job leasing에 transaction이 필요하다.
  3. desktop filtering/search/sort와 CLI query를 같은 storage 위에 통일해야 한다.

SQLite가 저장하는 것은 **metadata와 relationships**다.  
실제 skill contents는 filesystem artifact store에 남는다.

---

## 6. Core Domain Model

## 6.1 Identity rules

### 6.1.1 SourceId

`source_id = "src_" + sha256(canonical_locator)[0:16]`

Canonical locator examples:

- `github:owner/repo@ref#subdir`
- `local:/abs/path`
- `internal:drafts`
- `internal:collections`

### 6.1.2 SkillId

`skill_id = "skill::" + source_namespace + "/" + skill_slug`

- `source_namespace`는 source 기준 logical namespace다.
- imported skill과 internal draft skill이 충돌하지 않게 해야 한다.

### 6.1.3 Version IDs

Immutable content-addressed IDs:

- `skill_version_id = "sv_" + sha256(normalized_skill_directory_bytes)[0:16]`
- `deck_version_id = "dv_" + sha256(normalized_deck_definition)[0:16]`
- `guide_version_id = "gv_" + sha256(normalized_guide_text)[0:16]`
- `bundle_version_id = "bv_" + sha256(normalized_bundle_directory_bytes)[0:16]`
- `source_snapshot_id = "ss_" + sha256(source_resolved_state)[0:16]`

## 6.2 Entities

### 6.2.1 Source

| Field | Type | Required | Notes |
|---|---|---|---|
| id | string | yes | `SourceId` |
| kind | enum | yes | `github`, `local`, `internal` |
| canonical_locator | string | yes | parser-normalized source locator |
| display_name | string | yes | human label |
| default_ref | string/null | no | GitHub default branch or user pin |
| subdir | string/null | no | source subdirectory |
| auth_profile | string/null | no | future GitHub/private auth profile key |
| created_at | timestamp | yes | |
| updated_at | timestamp | yes | |
| archived | boolean | yes | soft hide only |

### 6.2.2 SourceSnapshot

| Field | Type | Required | Notes |
|---|---|---|---|
| id | string | yes | `SourceSnapshotId` |
| source_id | string | yes | FK Source |
| resolved_ref | string | yes | commit SHA or exact local hash reference |
| fetched_at | timestamp | yes | |
| content_hash | string | yes | normalized snapshot hash |
| cache_key | string/null | no | tar/unpack cache lookup |
| scan_status | enum | yes | `ok`, `warning`, `error` |
| warnings_json | json | yes | parser/discovery warnings |
| notes_json | json | yes | recipe/scan notes |
| import_mode | enum | yes | `manual`, `auto_sync`, `compat_install` |

### 6.2.3 Skill

Logical identity across versions.

| Field | Type |
|---|---|
| id | string |
| source_id | string |
| slug | string |
| title | string |
| description | string |
| category | string/null |
| tags_json | json |
| origin_kind | enum (`imported`, `draft`, `fork`, `augment`) |
| created_at | timestamp |
| updated_at | timestamp |
| archived | boolean |

### 6.2.4 SkillVersion

Immutable renderable/installable skill content.

| Field | Type | Notes |
|---|---|---|
| id | string | `SkillVersionId` |
| skill_id | string | FK Skill |
| source_snapshot_id | string/null | imported version이면 required |
| parent_skill_version_id | string/null | fork/augment lineage |
| content_hash | string | full hash |
| artifact_relpath | string | filesystem artifact root |
| schema_version | integer | praxis-native authoring schema version |
| frontmatter_json | json | normalized metadata |
| raw_body_relpath | string | usually `SKILL.md` |
| compatibility_json | json | toolchain compatibility / imported sidecars |
| created_at | timestamp | |
| created_by | enum | `import`, `create`, `fork`, `augment` |
| promotion_state | enum | `candidate`, `accepted`, `rejected`, `deprecated` |

### 6.2.5 GuideVersion

| Field | Type | Notes |
|---|---|---|
| id | string | `GuideVersionId` |
| source_snapshot_id | string/null | |
| source_id | string | |
| guide_kind | enum | `codex-agents`, `codex-override`, `codex-alias`, `claude-root`, `claude-dotclaude` |
| content_hash | string | |
| artifact_relpath | string | markdown file path |
| created_at | timestamp | |

### 6.2.6 Deck

Logical collection identity.

| Field | Type |
|---|---|
| id | string |
| source_id | string |
| slug | string |
| title | string |
| description | string/null |
| origin_kind | enum (`declared`, `synthesized`, `manual`) |
| created_at | timestamp |
| updated_at | timestamp |

### 6.2.7 DeckVersion

| Field | Type | Notes |
|---|---|---|
| id | string | `DeckVersionId` |
| deck_id | string | |
| source_snapshot_id | string/null | |
| membership_json | json | ordered list of `SkillId` or `SkillVersionId` references |
| recommendation_json | json | reason / badge / recipe hint |
| created_at | timestamp | |

### 6.2.8 RecipeBundleVersion

현재 코드의 recipe/bundle 개념을 보존하기 위한 internal entity.

| Field | Type |
|---|---|
| id | string |
| source_snapshot_id | string |
| source_id | string |
| bundle_slug | string |
| agent | enum (`codex`, `claude`) |
| artifact_relpath | string |
| metadata_json | json |
| created_at | timestamp |

### 6.2.9 Workspace

| Field | Type | Notes |
|---|---|---|
| id | string | `user` 또는 `repo:<hash>` |
| scope | enum | `user`, `repo` |
| root_path | string/null | repo scope이면 absolute path |
| settings_json | json | workspace-specific settings |
| created_at | timestamp | |
| updated_at | timestamp | |

### 6.2.10 WorkspaceInstall

Workspace가 원하는 install selection.

| Field | Type | Notes |
|---|---|---|
| id | string | |
| workspace_id | string | FK Workspace |
| source_id | string | unique within workspace |
| selected_skill_ids_json | json | logical selections |
| selected_deck_ids_json | json | logical selections |
| selected_guide_kinds_json | json | desired guides |
| excluded_skill_ids_json | json | explicit remove mask |
| update_policy | enum | `manual`, `follow_latest_source`, `pinned_snapshot` |
| pinned_source_snapshot_id | string/null | pinned mode |
| created_at | timestamp | |
| updated_at | timestamp | |

### 6.2.11 BenchmarkSuite

| Field | Type |
|---|---|
| id | string |
| title | string |
| description | string/null |
| target_kind | enum (`skill`, `deck`) |
| rubric_markdown | string |
| judge_policy_json | json |
| created_at | timestamp |
| updated_at | timestamp |

### 6.2.12 BenchmarkCase

| Field | Type |
|---|---|
| id | string |
| suite_id | string |
| title | string |
| prompt_markdown | string |
| fixtures_json | json |
| expected_signals_json | json |
| tags_json | json |
| order_index | integer |

### 6.2.13 BenchmarkRun

| Field | Type | Notes |
|---|---|---|
| id | string | |
| suite_id | string | |
| candidate_a_kind | enum | `skill_version`, `deck_version` |
| candidate_a_id | string | |
| candidate_b_kind | enum/null | optional A/B comparison |
| candidate_b_id | string/null | |
| judge_mode | enum | `ai`, `human_ab`, `hybrid` |
| status | enum | `queued`, `running`, `awaiting_human`, `succeeded`, `failed`, `cancelled` |
| aggregate_score | real/null | |
| promotion_recommendation | enum | `promote`, `hold`, `reject`, `manual_review` |
| summary_markdown | string/null | |
| started_at | timestamp/null | |
| ended_at | timestamp/null | |

### 6.2.14 ToolchainConnection

| Field | Type | Notes |
|---|---|---|
| id | string | |
| kind | enum | `codex`, `claude`, `gemini` |
| executable_path | string/null | |
| detected_version | string/null | |
| auth_state | enum | `unknown`, `not_required`, `missing`, `present`, `expired` |
| status | enum | `healthy`, `degraded`, `broken`, `unknown` |
| last_health_check_at | timestamp/null | |
| details_json | json | extra diagnostics |

### 6.2.15 Job

| Field | Type | Notes |
|---|---|---|
| id | string | |
| kind | enum | `source_sync`, `import_source`, `workspace_apply`, `benchmark_run`, `health_scan`, `augment_skill`, `export_skill` |
| payload_json | json | opaque typed payload |
| status | enum | `queued`, `leased`, `running`, `succeeded`, `failed`, `cancelled` |
| progress_json | json | percent, stage, counters |
| retry_count | integer | |
| max_retries | integer | |
| backoff_until | timestamp/null | |
| leased_by_session | string/null | desktop session or CLI worker id |
| lease_expires_at | timestamp/null | |
| error_code | string/null | |
| error_message | string/null | |
| created_at | timestamp | |
| started_at | timestamp/null | |
| ended_at | timestamp/null | |

---

## 7. Source Intake and Import Contract

## 7.1 Supported source kinds

1. `github`
2. `local`
3. `internal`

### 7.1.1 GitHub source canonicalization

Praxis는 current parser 규칙을 유지한다.

Accepted user input examples:

- full GitHub URL
- `owner/repo`
- GitHub URL with tree ref
- GitHub URL with subdirectory
- local absolute/relative path

Canonical source locator must normalize to:

```text
github:owner/repo@ref#subdir
local:/abs/path
internal:<name>
```

## 7.2 Scan roots

Inherited from current implementation. Importer must scan the following roots:

1. repository root skill
2. `skills/`
3. `.agents/skills/`
4. `.claude/skills/`

Ignored directories:

- `.git`
- `node_modules`
- `target`
- `.praxis`
- `.telos`

## 7.3 Safety constraints

1. tar unpack must reject symlink entries
2. tar unpack must reject hardlink entries
3. copied install content must reject symlink traversal
4. hash computation must reject symlink inputs
5. extracted paths must remain inside the intended destination root

## 7.4 Skill compatibility import contract

Praxis must continue to import current ecosystem skills accepted by the existing parser.

Observed required compatibility:

- `SKILL.md` exists
- YAML frontmatter is parseable
- `name` is validated against current slug regex
- directory name and declared skill name must agree
- description length/shape is validated
- optional sidecars may include:
  - `skill.json`
  - `agents/openai.yaml`

**Normative final decision**: praxis-native created skills use a superset schema but importer remains backward compatible with current source format.

## 7.5 Deck compatibility import contract

Importer must support:

1. declared deck file: `skills.deck.json`
2. synthesized deck generation:
   - `all`
   - by category
   - by prefix

Missing skill references in declared decks produce warnings or import errors according to strictness policy:

- default import mode: warning + deck marked invalid
- creator/export strict mode: error

## 7.6 Guide discovery contract

Importer must detect:

- `AGENTS.md`
- `AGENTS.override.md`
- optional `AGENT.md` alias
- `CLAUDE.md`
- `.claude/CLAUDE.md`

Guide assets are normalized into `GuideVersion` records with exact source path provenance.

## 7.7 Recipes

Recipe detection remains source-specific and optional.

Observed inherited behavior:
- `garrytan/gstack` recipe produces bundle/deck/guide recommendation output.

Normative final rule:
- recipe execution may only emit normalized internal entities:
  - `RecipeBundleVersion`
  - `DeckVersion`
  - recommendation notes
- recipe code must not write directly into runtime targets.

## 7.8 Import result

Successful import of a source snapshot must produce:

1. one `SourceSnapshot`
2. zero or more `Skill` + `SkillVersion`
3. zero or more `Deck` + `DeckVersion`
4. zero or more `GuideVersion`
5. zero or more `RecipeBundleVersion`
6. warnings/notes
7. searchable library metadata entries

---

## 8. Praxis-native Authoring Contract

## 8.1 Skill create/fork/augment

Praxis-native creation always produces:

1. logical `Skill`
2. immutable initial `SkillVersion`
3. artifact directory containing authoring files
4. provenance relation:
   - `create`: no parent version
   - `fork`: parent version required
   - `augment`: parent version required and must record augmentation prompt/context

## 8.2 Praxis-native `SKILL.md` superset schema

Praxis-created skills MUST emit this minimal frontmatter:

```yaml
---
schema_version: 1
name: debug-root-cause
title: Debug Root Cause
description: Find and verify the smallest root cause before patching.
category: debugging
tags:
  - debugging
  - root-cause
toolchains:
  - codex
  - claude
---
```

Rules:

- `schema_version` required for praxis-native created skills
- `name` must satisfy current slug regex
- `title` required for praxis-native created skills
- `description` required
- `category` optional
- `tags` optional but recommended
- `toolchains` optional; empty means “unspecified”

Importer compatibility mode must continue to accept older source skills without `schema_version` or `title`.

## 8.3 Draft editing model

A draft edit does **not** mutate an existing `SkillVersion`.
Instead:

1. load source version or latest draft version
2. edit working copy
3. validate
4. commit as new immutable `SkillVersion`
5. update `Skill.updated_at`

## 8.4 Rename semantics

Rename produces:

- same `Skill.id`
- new `SkillVersion`
- new slug only if user explicitly chooses slug rename
- deck memberships and workspace installs must be re-resolved on next plan/apply

## 8.5 Export semantics

Praxis must support exporting a skill or deck as filesystem content.
Export is a pure filesystem write and must not mutate the source-of-truth version record.

---

## 9. Library Plane Behavior

## 9.1 Library states

A skill or deck may display one or more derived library states:

- `Available`
- `Installed`
- `Imported`
- `Draft`
- `Augmented`
- `Outdated`
- `Benchmarked`
- `Recommended`

These are UI labels, not authoritative DB enums, except where separately stored.

Derived rules:

- `Installed` if referenced by any active `WorkspaceLock`
- `Imported` if latest version descends from `SourceSnapshot`
- `Draft` if latest version origin_kind is `draft`
- `Augmented` if latest version origin_kind is `augment`
- `Outdated` if an install points to older pinned snapshot while newer compatible snapshot exists
- `Benchmarked` if at least one completed `BenchmarkRun` exists
- `Recommended` if latest accepted benchmark/promotion recommends it

## 9.2 Search/filter/sort

Library query must support at minimum:

- text search over title/slug/description/tags/source
- filter by source
- filter by state
- filter by toolchain compatibility
- sort by updated_at, title, benchmark outcome, install count

## 9.3 Library item detail requirements

Each skill detail view must show:

1. latest version metadata
2. lineage (source snapshot or parent version)
3. benchmark summary
4. install state by workspace
5. deck memberships
6. available actions:
   - install
   - fork
   - augment
   - benchmark
   - export
   - inspect source

---

## 10. Deck Management Contract

## 10.1 Deck kinds

1. **Declared deck** — imported from source file
2. **Synthesized deck** — generated by importer (`all`, category, prefix)
3. **Manual deck** — created by user in praxis

## 10.2 Deck membership rules

- DeckVersion membership is ordered.
- Membership may reference `Skill.id`; resolution to concrete version happens at workspace planning time.
- Invalid references mark the deck invalid and block apply of that deck until resolved.

## 10.3 Manual collections

A manual deck is stored under internal source `internal:collections`.

Manual deck edits create a new immutable `DeckVersion`.

---

## 11. Runtime Plane: Workspace Contract

## 11.1 Workspace manifest v2

Format: TOML  
Location:
- repo scope: `<repo>/.praxis/manifest.toml`
- user scope: `<state_root>/workspace-user/manifest.toml` or equivalent platform-local manifest path

Canonical structure:

```toml
version = 2

[workspace]
scope = "repo" # or "user"
root_path = "/absolute/repo/path"

[settings]
claude_guide_location = "root"       # root | dotclaude
write_agent_alias = true

[[installs]]
source_id = "src_abcd1234ef567890"
update_policy = "manual"             # manual | follow_latest_source | pinned_snapshot
pinned_source_snapshot_id = ""

[installs.selection]
skills = ["skill::demo/debug-root-cause"]
decks = ["deck::demo/workflow"]
guides = ["codex-agents", "claude-root"]
exclude_skills = []
```

Rules:

1. one `source_id` may appear at most once per workspace manifest
2. `skills` and `decks` may coexist
3. `guides` is a list of desired guide outputs
4. `exclude_skills` masks skill membership inherited through deck selection
5. `pinned_source_snapshot_id` required only when `update_policy = "pinned_snapshot"`

## 11.2 Workspace lock v2

Format: JSON  
Location:
- repo scope: `<repo>/.praxis/lock.json`
- user scope: user workspace lock path

Canonical structure:

```json
{
  "version": 2,
  "workspace_id": "repo:5f2d8f...",
  "generated_at": "2026-03-16T00:00:00Z",
  "installs": [
    {
      "source_id": "src_abcd1234ef567890",
      "source_snapshot_id": "ss_0123abcd4567ef89",
      "resolved_ref": "a1b2c3d4e5f6",
      "skills": [
        {
          "skill_id": "skill::demo/debug-root-cause",
          "skill_version_id": "sv_1111222233334444",
          "agent": "codex",
          "target_path": ".agents/skills/debug-root-cause",
          "content_hash": "sha256:..."
        }
      ],
      "guides": [
        {
          "guide_kind": "codex-agents",
          "guide_version_id": "gv_99990000aaaa1111",
          "target_path": "AGENTS.md",
          "content_hash": "sha256:..."
        }
      ],
      "owned_paths": [
        ".agents/skills/debug-root-cause",
        "AGENTS.md"
      ]
    }
  ]
}
```

Lock is the authoritative record for owned runtime paths.

## 11.3 Planning algorithm

Given a workspace and current library state, planner must:

1. load workspace manifest
2. resolve each install to a `SourceSnapshot`
3. resolve selected decks to concrete skill set
4. apply exclusions
5. resolve concrete version IDs
6. discover runtime target paths
7. detect collisions
8. detect unmanaged conflicts
9. emit a pure `InstallPlan`

`InstallPlan` must be reviewable without side effects.

## 11.4 Apply algorithm

Apply must be atomic at the install transaction level.

Required sequence:

1. validate manifest and resolved versions
2. compute plan
3. if fatal conflicts exist, abort before filesystem writes
4. stage file copies
5. replace managed runtime targets
6. render and merge guides
7. prune stale owned paths that are no longer desired
8. write new lock
9. mark workspace install updated

## 11.5 Conflict model

Fatal conflict classes:

- `skill_collision`
- `bundle_collision`
- `unmanaged_conflict`
- `missing_version_artifact`
- `invalid_manifest`
- `guide_target_unwritable`

### 11.5.1 `skill_collision`

Raised when two active installs in the same workspace resolve to the same runtime skill name + agent target.

### 11.5.2 `unmanaged_conflict`

Raised when target path already exists and is not owned by current lock and is not explicitly replaced by user-approved migration flow.

## 11.6 Remove semantics

Removing an install must:

1. remove the `WorkspaceInstall` record from manifest
2. recompute desired runtime set
3. prune only paths owned by prior lock and no longer desired
4. preserve unrelated unmanaged files

## 11.7 Sync semantics

`sync` means:

1. refresh source snapshots according to update policy
2. re-resolve installs
3. recompute plan
4. apply if no fatal conflicts

`update` is an alias of sync in compatibility CLI mode.

---

## 12. Guide Rendering Contract

## 12.1 Guide target kinds

Inherited target kinds:

- `codex-agents`
- `codex-override`
- `codex-alias`
- `claude-root`
- `claude-dotclaude`

## 12.2 Managed block rule

Praxis must preserve user-authored guide content and only replace managed blocks.

Required marker family:

```html
<!-- praxis:begin ... -->
...
<!-- praxis:end -->
```

Normative final extension:
- v2 marker payload should be JSON metadata encoded after `praxis:begin`
- parser must remain backward compatible with existing v1-style markers

## 12.3 User content preservation

Guide apply algorithm:

1. read existing file if present
2. parse managed blocks
3. strip prior praxis-managed blocks
4. preserve remaining user-authored content
5. insert new managed blocks
6. write file

## 12.4 Alias handling

If `write_agent_alias = true`, praxis writes `AGENT.md` as alias output according to current workspace rules.  
If false, alias path is pruned when previously owned by lock.

---

## 13. Benchmark and Evaluation Contract

## 13.1 Benchmark modes

1. `ai`
2. `human_ab`
3. `hybrid`

## 13.2 Benchmark run lifecycle

States:

- `queued`
- `running`
- `awaiting_human`
- `succeeded`
- `failed`
- `cancelled`

Transitions:

1. create run → `queued`
2. worker lease acquired → `running`
3. if human comparison required → `awaiting_human`
4. decision complete → `succeeded`
5. hard failure → `failed`
6. operator cancel → `cancelled`

## 13.3 Candidate rules

A benchmark run must target either:

- one candidate (`candidate_a`) against suite expectations
- two candidates (`candidate_a`, `candidate_b`) for A/B comparison

Candidates may be:

- `SkillVersion`
- `DeckVersion`

## 13.4 AI judge contract

AI judge mode must persist:

1. full candidate outputs
2. judge prompt or rubric reference
3. judge result summary
4. case-level decisions
5. aggregate score

## 13.5 Human A/B contract

Human review mode must:

1. anonymize candidate labels as A/B
2. show prompt + outputs + rubric
3. capture vote
4. store reviewer note
5. support incomplete run resumption

## 13.6 Promotion recommendation

Each completed run computes:

- `promote`
- `hold`
- `reject`
- `manual_review`

Promotion recommendation is advisory until user confirms or auto-promote policy explicitly allows automation.

---

## 14. Connections and Health Contract

## 14.1 Toolchain kinds

- `codex`
- `claude`
- `gemini`

## 14.2 Health checks

At minimum praxis must check:

1. executable discoverability
2. version command success
3. auth presence/absence if applicable
4. runtime target path writability
5. guide path writability
6. workspace manifest/lock consistency
7. collision/conflict health

## 14.3 Health statuses

- `healthy`
- `degraded`
- `broken`
- `unknown`

## 14.4 Gemini scope

Normative final decision:
- Gemini integration belongs to Connections / Creator / Benchmark layers first.
- Runtime file install targets remain Codex/Claude until a concrete Gemini runtime file contract exists.

This keeps current runtime model precise without blocking broader integration.

---

## 15. Job Engine Contract

## 15.1 No daemon requirement

Praxis must **not require** a permanently running daemon.

Jobs are persisted, but work progresses only when:

- desktop session worker is active, or
- CLI worker command is active

## 15.2 Worker lease model

A worker acquires a lease on a queued job by writing:

- `leased_by_session`
- `lease_expires_at`
- status `leased` or `running`

If a session dies and lease expires, another worker may reclaim the job.

## 15.3 Retry policy

Job retry fields:

- `retry_count`
- `max_retries`
- `backoff_until`

Suggested defaults:

- source sync: 3 retries
- benchmark run: 1 retry for transient model/tool failure
- workspace apply: 0 automatic retries
- health scan: 1 retry

## 15.4 Job kinds

Required kinds:

- `source_sync`
- `import_source`
- `workspace_apply`
- `benchmark_run`
- `health_scan`
- `augment_skill`
- `export_skill`

---

## 16. CLI Contract

## 16.1 CLI posture

CLI is the authoritative automation interface.  
Desktop must call the same core operations.

## 16.2 Canonical command groups

```text
praxis source add <locator>
praxis source inspect <locator>
praxis source sync [--source <id>]

praxis library list
praxis library show <skill-or-deck-id>

praxis skill create
praxis skill fork <skill-version-id>
praxis skill augment <skill-version-id>
praxis skill export <skill-version-id> <path>

praxis deck create
praxis deck update
praxis deck export

praxis workspace init --scope repo|user
praxis workspace plan
praxis workspace apply
praxis workspace remove --source <id>
praxis workspace sync
praxis workspace doctor
praxis workspace guide show
praxis workspace guide set

praxis bench suite create
praxis bench run
praxis bench review
praxis bench promote

praxis connection list
praxis connection doctor

praxis jobs list
praxis jobs work
```

## 16.3 Compatibility aliases

Current CLI compatibility must be preserved initially.

| Current command | Canonical mapping |
|---|---|
| `praxis inspect` | `praxis source inspect` |
| `praxis install` | `praxis source add` + `praxis workspace apply` convenience path |
| `praxis remove` | `praxis workspace remove` |
| `praxis list` | `praxis workspace plan` or workspace/library summary |
| `praxis sync` | `praxis workspace sync` |
| `praxis update` | `praxis workspace sync` |
| `praxis doctor` | `praxis workspace doctor` |
| `praxis guidance ...` | `praxis workspace guide ...` |

## 16.4 Output contract

Every write or plan command must support machine-readable JSON output.

Required envelope:

```json
{
  "ok": true,
  "command": "workspace.plan",
  "data": {},
  "warnings": [],
  "errors": []
}
```

If `ok = false`, `errors` must contain stable error codes.

---

## 17. Desktop Contract

## 17.1 Top-level surfaces

Final desktop IA:

1. Discover
2. Library
3. Create
4. Benchmark Lab
5. Health
6. Connections
7. Settings

## 17.2 Global workspace context

Desktop must expose a global workspace selector:

- User Workspace
- Repo Workspace (selected folder)

Install state, plan preview, agent-file outputs, and doctor results are all scoped by this workspace context.

## 17.3 Plan review UX

The old dedicated Plan tab is replaced by a review surface/drawer/modal that can be invoked from:

- skill detail
- deck detail
- source import flow
- workspace overview

The plan itself remains a first-class concept; only navigation placement changes.

## 17.4 Agent Files UX

Agent Files are not a top-level nav item.
They appear in:

- workspace settings
- plan/apply review
- detail panel for installed workspace outputs

---

## 18. Validation and Error Surface

## 18.1 Validation classes

Required stable error codes:

- `source_parse_error`
- `unsupported_source_kind`
- `github_ref_resolution_error`
- `source_fetch_error`
- `unsafe_archive_entry`
- `invalid_skill_frontmatter`
- `invalid_skill_slug`
- `invalid_skill_directory_name`
- `invalid_deck_definition`
- `missing_skill_reference`
- `skill_collision`
- `bundle_collision`
- `unmanaged_conflict`
- `missing_version_artifact`
- `guide_target_unwritable`
- `workspace_manifest_invalid`
- `workspace_lock_invalid`
- `toolchain_not_found`
- `toolchain_auth_missing`
- `benchmark_suite_invalid`
- `job_lease_conflict`
- `migration_required`

## 18.2 Gating behavior

- Source import validation failures block only that import.
- Workspace manifest/plan validation failures block apply.
- Benchmark validation failures block run creation.
- Connection/health failures do not mutate library/runtime state unless explicitly part of apply gating.
- Invalid reload or partial import must not corrupt previous accepted library versions.

---

## 19. Migration Plan

## 19.1 Current-to-final upgrade rule

Current workspaces using manifest/lock v1 must remain readable.

Migration flow:

1. detect v1 workspace
2. read existing source installs from manifest/lock
3. create corresponding `Source` records in library DB
4. import current source snapshots
5. preserve existing runtime lock as source of ownership truth
6. on first successful v2 apply, write manifest v2 + lock v2

## 19.2 Backward compatibility window

Praxis must support:

- reading v1 manifest/lock
- writing v2 manifest/lock
- parsing current source skill format
- rendering current guide block markers

## 19.3 Non-destructive migration rule

Migration must never delete runtime-owned files before a successful v2 plan/apply cycle completes.

---

## 20. Acceptance Criteria

## 20.1 Source and library

1. A GitHub source can be added, scanned, and normalized into library items.
2. A local path source can be added and scanned equivalently.
3. Re-import of the same source produces a new `SourceSnapshot` without corrupting prior versions.
4. Invalid skill or deck definitions fail with stable error codes.

## 20.2 Runtime

1. A workspace can plan apply from library-resolved source installs.
2. Apply writes only managed targets and prunes only lock-owned stale targets.
3. Skill collision and unmanaged conflict are detected before writes.
4. Guide files preserve user-authored content across repeated apply.

## 20.3 Authoring

1. A skill can be created from scratch and appears in library as draft.
2. A skill can be forked from imported version and saved as new version.
3. Augment creates new immutable version with parent lineage.

## 20.4 Benchmark

1. A suite can be created with multiple cases.
2. A run can compare one or two candidates.
3. AI judge result and human review result persist and can be resumed after restart.
4. Promotion recommendation is visible in library detail.

## 20.5 Jobs

1. Queued jobs survive restart.
2. Stale leases can be reclaimed.
3. Failed jobs surface stable error codes and logs.

---

## 21. Delivery Phases

### Phase 1 — Library foundation
- SQLite store
- source import → snapshot → normalized versions
- compatibility CLI
- workspace integration using imported snapshots

### Phase 2 — UI re-architecture
- Discover / Library / workspace context
- plan review surface
- source/library detail views
- contextual deck and agent-file views

### Phase 3 — Authoring
- create/fork/augment
- export
- manual deck editing

### Phase 4 — Benchmark
- suites
- runs
- AI judge
- human A/B review
- promotion recommendation

### Phase 5 — Hardening
- private GitHub auth profiles
- richer health checks
- organization policy overlays
- extended recipe registry

---

## 22. Final Product Thesis

Praxis는 최종적으로 다음을 만족해야 한다.

- **source를 다루되 source에 갇히지 않는다**
- **skill을 저장하되 runtime install의 결정성을 잃지 않는다**
- **create와 benchmark를 추가하되 local-first 철학을 버리지 않는다**
- **desktop을 강화하되 core/CLI authoritative 구조를 유지한다**

이 문서의 가장 중요한 결론은 하나다.

> praxis의 최종형태는 “새 제품”이 아니라,  
> 이미 존재하는 deterministic source manager를 Runtime Plane으로 흡수한  
> **local-first skill workspace**다.
