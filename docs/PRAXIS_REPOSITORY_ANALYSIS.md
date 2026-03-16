# PRAXIS Repository Analysis Report

## 1. 결론

이 저장소의 **현재 구현 현실**과 **문서상 갱신 목표**는 명확히 구분된다.

- **현재 구현 현실**: `praxis`는 외부 GitHub/local repository를 스캔해 `SKILL.md`, `AGENTS.md`, `AGENTS.override.md`, `AGENT.md`, `CLAUDE.md`를 발견하고, 선택한 skill/deck/guide만 Codex/Claude 대상 경로에 복사 설치하는 **GitHub-first source manager**다.
- **문서상 갱신 목표**: `docs/specs/renewal`은 praxis를 discovery/create/import/augmentation/deck management/benchmarking이 1급인 **task-oriented skill workspace**로 확장한다.
- **핵심 판단**: 이 둘은 서로 대체 관계가 아니라 **층위 관계**다. 현재 `praxis-core`가 가진 deterministic installer/reconciler를 버리면 안 된다. 그것을 최종 제품의 **Runtime Plane**으로 보존하고, 그 위에 Library / Creation / Benchmarking을 올리는 것이 가장 일관되고 구현 가능하다.

즉, 이 레포의 정답은 “지금 있는 source manager를 버리고 새 제품을 따로 만드는 것”이 아니라, **현재 구현을 하위 코어로 흡수한 로컬 우선 skill workspace**로 확정하는 것이다.

## 2. 분석 방법과 증거 등급

이 문서는 다음 규칙으로 작성했다.

- **FACT-CODE**: 현재 저장소 코드 파일을 직접 읽어 확인한 사실.
- **FACT-DOC**: 저장소 문서/예제/계획 문서에서 직접 확인한 사실.
- **FACT-ASSET**: GitHub 트리에서 존재를 확인한 정적 자산.
- **DECISION**: 위 사실을 보존하면서 최종 제품을 위해 내가 명시적으로 선택한 설계 결정.
- **GAP**: 현재 코드와 갱신 문서 사이의 차이.

추정은 사용하지 않았다. 미래 상태에 대한 내용은 모두 **설계 결정**으로 분리했다.

## 3. 저장소 현실 요약

### 3.1 현재 구현의 본질

현재 코드베이스에서 가장 중요한 구현 사실은 다음이다.

1. **코어가 진짜 제품**이다. Tauri desktop은 얇은 command bridge이고, 상태 모델·source scan·plan·install·guide merge·doctor는 거의 모두 `crates/praxis-core`에 모여 있다.
2. **설치 모델은 copy-only + lock-driven reconciliation**이다.
3. **현재 제품은 source-centric**이다. 사용자는 source를 inspect하고, skill/deck/guide selection을 만들고, plan을 본 뒤 install/apply한다.
4. **guide 관리가 별도 서브시스템으로 완성돼 있다**. 사용자 작성 내용과 praxis managed block을 분리해 보존한다.
5. **보안/안전 장치가 이미 존재한다**. tar unpack과 copy 단계에서 symlink/hardlink를 거부하고, unmanaged conflict와 ownership을 검사한다.

### 3.2 문서상 갱신 목표의 본질

`docs/specs/renewal`이 보여주는 목표는 다음이다.

1. 제품 중심이 source install UI에서 **skill workspace**로 이동한다.
2. 주요 surface가 Discover / My Skills / Create / Decks / Benchmark Lab으로 재구성된다.
3. GitHub source는 더 이상 최종 목적이 아니라 **library intake origin**이 된다.
4. guide는 1급 navigation에서 내려오고, contextual/runtime output이 된다.
5. benchmark와 evaluation이 promotion 판단의 핵심 축이 된다.

### 3.3 가장 중요한 구조적 사실

이 레포에는 사실상 두 개의 truth layer가 있다.

- **Layer A — 구현 truth**: `README`, `docs/01-09`, `praxis-core`, `praxis-cli`, `praxis-desktop`
- **Layer B — 목표 truth**: `docs/specs/renewal`, `plans/IMPLEMENTATION-PLAN.md`, `plans/TASKS.md`

최종 스펙은 Layer B만 따라가면 안 되고, Layer A의 작동하는 불변조건을 보존해야 한다.

## 4. 핵심 설계 판단

### 4.1 최종형태 확정

**DECISION**: 최종 제품은 다음의 3-plane 구조로 확정한다.

1. **Library Plane**  
   가져온 skill/deck/source version, draft, augmented variant, provenance, benchmark 상태를 보관하는 로컬 라이브러리.

2. **Runtime Plane**  
   현재 `praxis-core`가 이미 구현한 deterministic install/apply/reconcile/doctor/guidance 엔진.  
   최종 제품에서도 이 plane은 유지된다.

3. **Evaluation Plane**  
   benchmark suite/run, AI judge, human A/B, promotion decision을 담당하는 평가 계층.

보조 surface는 Connections / Health / Settings다.

### 4.2 버리면 안 되는 현재 불변조건

다음은 최종 제품에서도 **반드시 유지**해야 한다.

- copy-only install
- lock ownership
- deterministic stale prune
- unmanaged overwrite 금지
- source scan → plan → apply 분리
- guide user content 보존
- CLI authoritative posture
- 로컬 우선 / no required cloud backend

### 4.3 현재 코드에서 드러난 강점

- 코어 경계가 이미 명확하다.
- 데이터 이동이 파일/복사/lock 관점으로 결정적이다.
- doctor와 conflict 모델이 존재한다.
- guide merge가 실사용 가능한 수준이다.

### 4.4 현재 코드에서 드러난 한계

- library 개념이 없다.
- source snapshot / local version / benchmark result를 저장할 영속 모델이 없다.
- UI가 `App.svelte` 단일 대형 컴포넌트에 집중돼 있다.
- source-centric CLI/desktop이라 renewal IA와 직접 대응되지 않는다.
- background job과 long-running task model이 없다.
- recipe 시스템이 `gstack` 단일 recipe에 머무른다.

## 5. GAP 분석

| 영역 | 현재 코드 | renewal 문서 | 판단 |
|---|---|---|---|
| 제품 중심 | source install manager | skill workspace | source manager를 Runtime Plane으로 흡수해야 함 |
| 데이터 모델 | source install + manifest/lock 중심 | library / version / benchmark / job 중심 | 새로운 library metadata 계층 필요 |
| UI IA | Catalog / Plan / Installed / Guides / Doctor / Settings | Discover / My Skills / Create / Decks / Benchmark Lab | UI 재구성 필요, 단 plan/apply 기능은 유지 |
| guide 위치 | 1급 탭 | contextual output | guide editor를 workspace context로 이동 |
| background jobs | 없음 | 필요 | resident daemon 없이 persisted cooperative jobs 채택 |
| 평가 | 없음 | benchmark/evaluation 1급 | Evaluation Plane 신설 필요 |

## 6. 파일 단위 분석

아래는 저장소에서 확인된 파일을 하나씩 정리한 인벤토리다.  
정적 아이콘 자산은 존재/역할이 명확하고 제품 로직이 없으므로 그 사실을 그대로 기록했다.

## 6.1 ROOT files

| Path | Kind | Evidence | Analysis |
|---|---|---|---|
| `.gitignore` | config | FACT-CODE | Git, Rust, Node/Vite, Tauri 산출물과 `.praxis/` 상태 디렉터리를 제외한다. 개발/빌드 노이즈를 저장소에서 배제하는 역할이다. |
| `Cargo.lock` (5,572 lines (GitHub metadata)) | lockfile | FACT-DOC | 워크스페이스 Rust 의존성 해상도 스냅샷이다. GitHub 파일 메타데이터상 5,572 lines / 133 KB이며 제품 로직이 아니라 재현 가능한 빌드를 위한 잠금 파일이다. |
| `Cargo.toml` | workspace manifest | FACT-CODE | Rust workspace 루트. 멤버는 `crates/praxis-core`, `crates/praxis-cli`, `apps/praxis-desktop/src-tauri`이고 workspace version은 `1.1.0`이다. |
| `README.md` | product doc | FACT-DOC | 현재 제품을 `GitHub-first management plane for agent skills and agent guidance`로 규정한다. 좁은 제품 경계, 사용법, 배포 자세, 상태를 한곳에 요약한다. |

## 6.2 docs files

| Path | Kind | Evidence | Analysis |
|---|---|---|---|
| `docs/01-PRODUCT.md` | doc | FACT-DOC | 제품 논지와 범위를 정의한다. praxis는 외부 skill 소스와 가이드를 설치/업데이트/제거/동기화하는 좁은 관리 평면이며 authoring framework/registry/cloud가 아님을 명시한다. |
| `docs/02-ARCHITECTURE.md` | doc | FACT-DOC | 현재 아키텍처의 기준 문서다. `praxis-core`/`praxis-cli`/`praxis-desktop` 분리, inspect→plan→apply 흐름, copy-only/lock-driven reconciliation, background daemon 비채택 등을 설명한다. |
| `docs/03-MODEL.md` | doc | FACT-DOC | 현재 도메인 모델을 정리한다. SourceRef, catalog, skill/deck/guide 자산, selection, manifest/lock, recipes, guide targets 등 현재 구현과 직접 대응되는 모델 문서다. |
| `docs/04-CLI.md` | doc | FACT-DOC | CLI 명령 계약을 정의한다. init/inspect/plan/install/remove/list/sync/update/doctor 및 guidance 하위 명령의 역할과 사용 패턴을 설명한다. |
| `docs/05-UI.md` | doc | FACT-DOC | 현재 데스크톱 UI 정보구조를 정의한다. Catalog, Plan, Installed, Guides, Doctor, Settings 탭과 inspect→plan→apply 보조 UX를 설명한다. |
| `docs/06-ROADMAP.md` | doc | FACT-DOC | v1 전후 기능의 단계적 진화를 설명한다. 현재 제품 이후에 추가할 검증/인증/조직 정책 등의 방향을 나열한다. |
| `docs/07-RELEASES.md` | doc | FACT-DOC | 릴리즈 패키지 관점의 운영 문서다. source package, CLI archive, macOS app bundle 같은 배포 산출물 구성을 설명한다. |
| `docs/08-TASKS.md` | doc | FACT-DOC | 작업 분해 및 잔여 구현 항목 문서다. 구현 순서와 해야 할 일의 체크리스트 역할을 한다. |
| `docs/09-DEPLOYMENT.md` | doc | FACT-DOC | 배포 전략 문서다. 로컬 우선 패키징, GitHub Actions 자동 릴리즈 비채택, auto-update 비채택 같은 배포 철학을 명시한다. |
| `docs/specs/renewal/01-product-renewal-overview.md` | doc | FACT-DOC | 제품 갱신의 전체 논지와 scope guardrail을 설명한다. 좁은 source manager에서 더 넓은 task-oriented skill workspace로의 전환을 정의한다. |
| `docs/specs/renewal/02-information-architecture-and-ux.md` | doc | FACT-DOC | 갱신 제품의 정보구조와 화면 계층을 정의한다. Discover, My Skills, Create, Decks, Benchmark Lab 등 상위 surface의 배치를 설명한다. |
| `docs/specs/renewal/03-skill-library-and-deck-management.md` | doc | FACT-DOC | 로컬 skill library와 deck 관리 규칙을 정의한다. Installed/Imported/Draft/Augmented/Outdated/Benchmarked/Recommended 같은 상태 언어의 근거다. |
| `docs/specs/renewal/04-github-intake-and-sync.md` | doc | FACT-DOC | GitHub intake, source import, source-to-library mapping, sync/update 흐름을 정의한다. |
| `docs/specs/renewal/05-skill-creator-and-augmentation.md` | doc | FACT-DOC | skill 생성, fork, rename, augment, agent-assisted editing 경계를 정의한다. |
| `docs/specs/renewal/06-benchmark-lab-and-evaluation.md` | doc | FACT-DOC | AI judge, human A/B, benchmark trigger, acceptance/promotion 로직의 경계를 정의한다. |
| `docs/specs/renewal/07-toolchain-integrations-and-settings.md` | doc | FACT-DOC | Codex/Claude/Gemini 연결, 설정, health 영역을 정의한다. |
| `docs/specs/renewal/08-domain-model-and-background-jobs.md` | doc | FACT-DOC | 갱신 제품의 영속 엔티티와 background job orchestration을 정의한다. |
| `docs/specs/renewal/README.md` | doc | FACT-DOC | renewal spec set의 인덱스다. 현재 제품은 source manager이고 renewal target은 discovery/create/import/augmentation/decks/benchmarking이 1급인 skill workspace라고 명시한다. |

## 6.3 examples files

| Path | Kind | Evidence | Analysis |
|---|---|---|---|
| `examples/composable-skills.skills.deck.json` | example | FACT-DOC | deck 선언 예시 파일이다. skill 묶음을 deck으로 표현하는 JSON 계약의 예제로 쓰인다. |
| `examples/manifest.repo.sample.toml` | example | FACT-DOC | repo scope manifest 예시 파일이다. 현재 설치 선택 구조를 TOML로 어떻게 기록하는지 보여준다. |
| `examples/sources/demo-cards/AGENTS.md` | example | FACT-DOC | 예제 source의 Codex 가이드 파일이다. 가이드 스캔/설치/관리 대상이 실제로 어떤 파일인지 보여준다. |
| `examples/sources/demo-cards/CLAUDE.md` | example | FACT-DOC | 예제 source의 Claude 가이드 파일이다. Claude guide discovery와 apply 대상으로 쓰인다. |
| `examples/sources/demo-cards/debug-root-cause/SKILL.md` | example | FACT-DOC | 예제 skill 카드. root cause debugging 워크플로를 설명하는 frontmatter+body 구조 예시다. |
| `examples/sources/demo-cards/plan-cleanly/SKILL.md` | example | FACT-DOC | 예제 skill 카드. 계획 수립 워크플로를 설명하는 frontmatter+body 구조 예시다. |
| `examples/sources/demo-cards/ship-checklist/SKILL.md` | example | FACT-DOC | 예제 skill 카드. 릴리즈 전 체크리스트 워크플로를 설명하는 예시다. |
| `examples/sources/demo-cards/skills.deck.json` | example | FACT-DOC | 예제 source가 선언한 deck 파일이다. scan 시 declared deck으로 읽힌다. |
| `examples/sources/demo-cards/workflow-build-release/SKILL.md` | example | FACT-DOC | 예제 skill 카드. build/release workflow를 설명하는 예시다. |

## 6.4 plans files

| Path | Kind | Evidence | Analysis |
|---|---|---|---|
| `plans/IMPLEMENTATION-PLAN.md` | ops | FACT-DOC | renewal target을 실제 구현 단위로 전개하는 계획 문서다. 현재 코드와 갱신 스펙 사이의 중간 계획을 제공한다. |
| `plans/TASKS.md` | ops | FACT-DOC | 구현 태스크 목록이다. 실제 작업 분해와 진행 순서를 나타낸다. |

## 6.5 scripts files

| Path | Kind | Evidence | Analysis |
|---|---|---|---|
| `scripts/package-source.sh` | ops | FACT-DOC | 감사 가능한 source package를 만드는 로컬 릴리즈 스크립트다. |
| `scripts/release-cli.sh` | ops | FACT-DOC | CLI 배포 아카이브를 만드는 로컬 스크립트다. |
| `scripts/release-macos.sh` | ops | FACT-DOC | macOS 앱 번들을 패키징하는 로컬 스크립트다. |

## 6.6 crates files

| Path | Kind | Evidence | Analysis |
|---|---|---|---|
| `crates/praxis-cli/Cargo.toml` | code | FACT-CODE | CLI crate manifest. `praxis` 바이너리의 빌드 계약이다. |
| `crates/praxis-cli/src/main.rs` (266 lines) | code | FACT-CODE | CLI 진입점이다. clap 기반 명령 파싱, JSON 출력, scope/agent/guide 인자 처리, core manager 호출을 수행한다. |
| `crates/praxis-core/Cargo.toml` | code | FACT-CODE | core crate manifest. 현재 제품의 핵심 로직이 모이는 crate 의존성과 메타데이터를 선언한다. |
| `crates/praxis-core/src/guidance.rs` (324 lines) | code | FACT-CODE | guide 관리 엔진이다. managed block 파싱/렌더링, 사용자 작성 내용 보존, AGENT alias 처리, guide apply/remove를 담당한다. |
| `crates/praxis-core/src/lib.rs` | code | FACT-CODE | core public module export 집합이다. guidance/manager/model/parser/recipes/source/workspace를 re-export한다. |
| `crates/praxis-core/src/manager.rs` (1,248 lines) | code | FACT-CODE | 현재 제품의 orchestration 중심 파일이다. init/list/inspect/plan/install/remove/sync/update/doctor를 구현하고 collision/unmanaged conflict/stale prune/lock write를 지휘한다. |
| `crates/praxis-core/src/model.rs` (384 lines) | code | FACT-CODE | 현재 제품의 핵심 타입 정의 파일이다. Agent, Scope, SourceRef, catalog, manifest/lock, guide 상태, doctor/plan/install 요청·응답 타입을 보유한다. |
| `crates/praxis-core/src/parser.rs` (216 lines) | code | FACT-CODE | 입력 파서 계층이다. GitHub/local source 식별자 파싱, canonical source id 생성, `SKILL.md` frontmatter 검증, sidecar JSON/YAML 스키마 파싱을 수행한다. |
| `crates/praxis-core/src/recipes.rs` (66 lines) | code | FACT-CODE | source-specific recipe 계층이다. 현재 내장 recipe는 `garrytan/gstack` 전용이며 bundle/deck/guide recommendation을 합성한다. |
| `crates/praxis-core/src/source.rs` (650 lines) | code | FACT-CODE | source intake와 scan 엔진이다. GitHub tarball fetch, 안전한 unpack, skill/deck/guide/recipe discovery, hash 계산, 경고/노트 생성이 집중되어 있다. |
| `crates/praxis-core/src/workspace.rs` (180 lines) | code | FACT-CODE | repo/user scope 경로 해상도 계층이다. `.praxis`, `.agents/skills`, `.claude/skills`, guide 파일 경로, manifest/lock 초기화를 담당한다. |

## 6.7 apps files

| Path | Kind | Evidence | Analysis |
|---|---|---|---|
| `apps/praxis-desktop/index.html` | code/config | FACT-CODE | Vite/Tauri 프론트엔드 진입 HTML이다. 앱 타이틀과 mount 포인트를 정의한다. |
| `apps/praxis-desktop/package-lock.json` | code/config | FACT-CODE | Node 의존성 잠금 파일이다. 프런트엔드/Tauri JS 의존성 재현성을 위한 generated artifact다. |
| `apps/praxis-desktop/package.json` | code/config | FACT-CODE | 프런트엔드/Tauri JS 패키지 manifest다. `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `lucide-svelte`, `svelte`, `vite`, `typescript` 등을 선언한다. |
| `apps/praxis-desktop/src-tauri/Cargo.toml` | code/config | FACT-CODE | Tauri backend crate manifest다. `praxis-core`와 `tauri-plugin-dialog`를 연결한다. |
| `apps/praxis-desktop/src-tauri/build.rs` | code/config | FACT-CODE | Tauri 빌드 스크립트다. `tauri_build::build()` 호출만 수행한다. |
| `apps/praxis-desktop/src-tauri/capabilities/default.json` | code/config | FACT-CODE | Tauri capability 선언이다. core 기본 capability와 dialog open 권한을 허용한다. |
| `apps/praxis-desktop/src-tauri/icons/128x128.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/128x128@2x.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/32x32.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/64x64.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/Square107x107Logo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/Square142x142Logo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/Square150x150Logo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/Square284x284Logo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/Square30x30Logo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/Square310x310Logo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/Square44x44Logo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/Square71x71Logo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/Square89x89Logo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/StoreLogo.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-anydpi-v26/ic_launcher.xml` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-hdpi/ic_launcher.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-hdpi/ic_launcher_foreground.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-hdpi/ic_launcher_round.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-mdpi/ic_launcher.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-mdpi/ic_launcher_foreground.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-mdpi/ic_launcher_round.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-xhdpi/ic_launcher.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-xhdpi/ic_launcher_foreground.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-xhdpi/ic_launcher_round.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxhdpi/ic_launcher.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxhdpi/ic_launcher_foreground.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxhdpi/ic_launcher_round.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxxhdpi/ic_launcher.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxxhdpi/ic_launcher_foreground.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/mipmap-xxxhdpi/ic_launcher_round.png` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/android/values/ic_launcher_background.xml` | asset | FACT-ASSET | Android launcher/icon 리소스다. 해상도별 번들 자산이며 제품 로직은 없다. |
| `apps/praxis-desktop/src-tauri/icons/icon.icns` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/icon.ico` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/icon.png` | asset | FACT-ASSET | 플랫폼 번들링용 정적 아이콘 자산이다. 제품 로직은 없고 패키징/OS 표시에만 쓰인다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-20x20@1x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-20x20@2x-1.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-20x20@2x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-20x20@3x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-29x29@1x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-29x29@2x-1.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-29x29@2x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-29x29@3x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-40x40@1x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-40x40@2x-1.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-40x40@2x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-40x40@3x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-512@2x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-60x60@2x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-60x60@3x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-76x76@1x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-76x76@2x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/icons/ios/AppIcon-83.5x83.5@2x.png` | asset | FACT-ASSET | iOS 앱 아이콘 자산이다. 제품 로직은 없고 플랫폼 패키징/표시에만 사용된다. |
| `apps/praxis-desktop/src-tauri/src/lib.rs` (101 lines) | code/config | FACT-CODE | Tauri command bridge다. `workspace`, `inspect`, `plan`, `install`, `remove_install`, `sync`, `update`, `doctor`, `guidance`, `guidance_write` 명령을 노출한다. |
| `apps/praxis-desktop/src-tauri/src/main.rs` | code/config | FACT-CODE | Tauri 런타임 진입점이다. `praxis_desktop_lib::run()`을 호출한다. |
| `apps/praxis-desktop/src-tauri/tauri.conf.json` | code/config | FACT-CODE | Tauri 앱 번들 설정이다. productName `praxis`, identifier `com.axiomorient.praxis`, version `1.1.0`, macOS bundle targets를 정의한다. |
| `apps/praxis-desktop/src/App.svelte` (1,440 lines) | code/config | FACT-CODE | 현재 데스크톱 UI의 대부분을 담는 단일 루트 컴포넌트다. 탭 전환, inspect/plan/install/remove/sync/update/doctor/guidance 저장 흐름과 파생 상태를 모두 보유한다. |
| `apps/praxis-desktop/src/app.css` (1,342 lines) | code/config | FACT-CODE | 데스크톱 쉘 전체 스타일 시트다. 레이아웃, 카드, 패널, 탭, 폼, 상태 배지 스타일을 포괄한다. |
| `apps/praxis-desktop/src/lib/api.ts` (53 lines) | code/config | FACT-CODE | Tauri `invoke` 래퍼 계층이다. backend command와 프런트엔드 타입 사이의 얇은 연결만 담당한다. |
| `apps/praxis-desktop/src/lib/components/Card.svelte` | code/config | FACT-CODE | 재사용 가능한 일반 카드 컴포넌트다. |
| `apps/praxis-desktop/src/lib/components/DeckCard.svelte` | code/config | FACT-CODE | deck 시각화/선택 컴포넌트다. 추천/declared/synthesized 배지와 preview skill 목록을 표현한다. |
| `apps/praxis-desktop/src/lib/components/GuideEditor.svelte` | code/config | FACT-CODE | guide user content 편집 컴포넌트다. managed block 수와 metadata를 표시하고 저장 콜백을 트리거한다. |
| `apps/praxis-desktop/src/lib/components/InstalledSourceCard.svelte` | code/config | FACT-CODE | 설치된 source 요약 카드 컴포넌트다. target, selection, ref, hash, excluded/guides 정보를 보여준다. |
| `apps/praxis-desktop/src/lib/components/StarterSourceCard.svelte` | code/config | FACT-CODE | starter source 프리셋 카드 컴포넌트다. inspect/open 액션과 featured 배지를 제공한다. |
| `apps/praxis-desktop/src/lib/i18n/en.ts` (213 lines) | code/config | FACT-CODE | 영문 UI 사전이다. 현재 IA와 탭/버튼/설명 문구를 가장 직접적으로 드러내는 텍스트 자산이다. |
| `apps/praxis-desktop/src/lib/i18n/index.ts` | code/config | FACT-CODE | 로케일 선택, localStorage persistence, 번역 lookup을 담당하는 i18n 인프라다. |
| `apps/praxis-desktop/src/lib/i18n/ja.ts` (213 lines) | code/config | FACT-CODE | 일문 UI 사전이다. 영문 키셋과 동일한 구조를 유지한다. |
| `apps/praxis-desktop/src/lib/i18n/ko.ts` (213 lines) | code/config | FACT-CODE | 국문 UI 사전이다. 영문 키셋과 동일한 구조를 유지한다. |
| `apps/praxis-desktop/src/lib/starterSources.ts` | code/config | FACT-CODE | starter source 프리셋 목록과 매칭 로직을 담는다. anthropics/skills, open-skills, codex-skills, claude-skills를 내장한다. |
| `apps/praxis-desktop/src/lib/types.ts` (265 lines) | code/config | FACT-CODE | 프런트엔드 타입 정의 파일이다. backend model을 거의 1:1로 미러링한다. |
| `apps/praxis-desktop/src/main.ts` | code/config | FACT-CODE | Svelte 앱 마운트 진입점이다. |
| `apps/praxis-desktop/tsconfig.json` | code/config | FACT-CODE | TypeScript 컴파일 옵션과 strictness를 정의한다. |
| `apps/praxis-desktop/vite.config.ts` | code/config | FACT-CODE | Vite 빌드 설정이다. Tauri 개발 호스트, 브라우저 target, release minify 정책을 정의한다. |

## 7. 코드 수준 핵심 관찰

### 7.1 `praxis-core`가 실제 제품 엔진이다

- `manager.rs`가 init/list/inspect/plan/install/remove/sync/update/doctor를 모두 오케스트레이션한다.
- `source.rs`가 GitHub/local intake, tarball fetch/unpack, skill/deck/guide scan, hashing을 담당한다.
- `guidance.rs`가 managed block merge를 담당한다.
- `workspace.rs`가 repo/user scope target path를 결정한다.
- `parser.rs`가 source canonicalization과 `SKILL.md` frontmatter validation을 담당한다.

이 구조는 최종 제품에서도 유지해야 한다. UI나 CLI를 갈아엎어도 core orchestration을 중심에 두어야 한다.

### 7.2 Tauri backend는 얇다

`apps/praxis-desktop/src-tauri/src/lib.rs`는 command bridge일 뿐이다.  
즉, 데스크톱 앱은 제품 로직의 중심이 아니라 **입력/시각화 레이어**다.  
이 사실은 이후에도 “CLI/core authoritative, desktop assistive” 원칙을 유지해야 한다는 강한 근거다.

### 7.3 프런트엔드의 가장 큰 구조 문제는 `App.svelte` 집중이다

현재 UI는 동작하지만, 상태/행위/뷰 파생 로직이 한 파일로 과도하게 집중되어 있다.  
renewal IA를 구현하려면 surface별 route/component/store 단위로 분해해야 한다.

## 8. 최종형태에 대한 확정 판단

다음 선택을 최종안으로 확정한다.

1. praxis는 **로컬 우선 skill workspace**다.
2. source import는 최종 목적이 아니라 **library intake**다.
3. 현재 installer/reconciler는 **Runtime Plane**으로 보존한다.
4. library metadata는 runtime manifest/lock과 분리된 **별도 영속 저장소**를 가진다.
5. benchmark 결과는 promotion 판단의 정식 입력이 된다.
6. background job은 필요하지만, 현재 철학을 보존하기 위해 **resident daemon이 아니라 persisted cooperative job worker**로 설계한다.
7. CLI는 자동화의 기준 구현이며, desktop은 이를 시각화하고 보조한다.

## 9. 구현 우선순위

가장 안전한 순서는 다음이다.

1. **Library metadata 계층 추가**
2. **source import → library mapping 구현**
3. **runtime manifest/lock을 library-aware 하게 확장**
4. **desktop IA 재편**
5. **skill create/fork/augment**
6. **benchmark lab**
7. **promotion / recommendation / health 고도화**

## 10. 이 분석을 바탕으로 만든 산출물

이 분석 리포트와 함께 아래 두 문서를 별도 작성했다.

- `PRAXIS_FINAL_SPEC.md` — 실제 구현 가능한 단일 canonical spec
- `PRAXIS_SPEC_WRITING_STANDARD.md` — codex skill 제작을 위한 spec 작성 표준

이 두 문서는 현재 코드의 불변조건을 잃지 않으면서 renewal 목표를 실구현 가능한 수준으로 압축한 결과물이다.