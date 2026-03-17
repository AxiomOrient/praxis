<script lang="ts">
  import { fade, slide } from "svelte/transition";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    Activity,
    ArrowRight,
    BookOpen,
    CheckCircle,
    DownloadCloud,
    Folder,
    Layers,
    RefreshCw,
    Search,
    Settings2,
    Sparkles,
    Trash2,
    User,
  } from "lucide-svelte";
  import Card from "./lib/components/Card.svelte";
  import DeckCard from "./lib/components/DeckCard.svelte";
  import AgentFileEditor from "./lib/components/AgentFileEditor.svelte";
  import InstalledSourceCard from "./lib/components/InstalledSourceCard.svelte";
  import StarterSourceCard from "./lib/components/StarterSourceCard.svelte";
  import {
    augmentCreateDraft,
    agentFiles,
    agentFilesWrite,
    benchmarkRun,
    cancelJob,
    createSkillDraft,
    doctor,
    forkCreateDraft,
    inspect,
    install,
    plan,
    previewCreateDraft,
    promoteCreateDraft,
    remove,
    submitHumanReview,
    sync,
    retryJob,
    update,
    updateCreateDraft,
    workJobs,
    workspace,
  } from "./lib/api";
  import type {
    Agent,
    AgentFileSlot,
    AgentFileSnapshot,
    AgentFileTemplate,
    AppliedInstall,
    BenchmarkRunSummary,
    DoctorReport,
    DraftPreview,
    InstallPlan,
    InstallPayload,
    JobSummary,
    Scope,
    SkillInfo,
    SourceCatalog,
    SourceRef,
    WorkspaceSettings,
    WorkspaceSnapshot,
  } from "./lib/types";
  import { LANGUAGE_OPTIONS, loadLocale, saveLocale, translate, type Locale } from "./lib/i18n";
  import { DEFAULT_STARTER_SOURCE, matchStarterSource, STARTER_SOURCES } from "./lib/starterSources";

  const AGENT_FILE_SLOTS: AgentFileSlot[] = [
    "codex-project-root",
    "codex-project-override",
    "codex-user-root",
    "codex-user-override",
    "claude-project-root",
    "claude-project-dot",
    "claude-user-root",
  ];
  const STARTER_DECK_IDS = ["core", "starter", "default", "workflow"];

  let scope = $state<Scope>("user");
  let workspaceRoot = $state("");
  let sourceInput = $state(DEFAULT_STARTER_SOURCE.url);
  let catalog = $state<SourceCatalog | null>(null);
  let snapshot = $state<WorkspaceSnapshot | null>(null);
  let agentFileSnapshot = $state<AgentFileSnapshot | null>(null);
  let report = $state<DoctorReport | null>(null);
  let planState = $state<InstallPlan | null>(null);
  let selectedDecks = $state<string[]>([]);
  let selectedSkills = $state<string[]>([]);
  let selectedAgentFileTemplates = $state<string[]>([]);
  let excludedSkills = $state<string[]>([]);
  let targets = $state<Agent[]>(["codex", "claude"]);
  let allSkills = $state(false);
  let activeTab = $state<
    "discover" | "plan" | "library" | "agent-files" | "create" | "benchmarks" | "connections" | "health" | "settings"
  >("discover");
  let activeAgentFileSlot = $state<AgentFileSlot>("codex-project-root");
  let deckQuery = $state("");
  let skillQuery = $state("");
  let agentFileTemplateQuery = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let showAdvancedCatalog = $state(false);
  let showAdvancedWorkspace = $state(false);
  let locale = $state<Locale>(loadLocale());
  let benchmarkResult = $state<BenchmarkRunSummary | null>(null);
  let createPreview = $state<DraftPreview | null>(null);
  let selectedDraftId = $state("");
  let draftName = $state("");
  let draftDescription = $state("Draft created from Praxis desktop flow.");
  let draftPreset = $state("skill");
  let benchmarkMode = $state("deterministic");
  let aiExecutorModel = $state("");
  let reviewDecision = $state("promote");
  let reviewNote = $state("");
  let augmentPrompt = $state("Tighten the purpose, inputs, and outputs.");
  let selectedForkSkill = $state("");
  let draftEditorPath = $state("SKILL.md");
  let draftEditorContent = $state("");

  function tr(
    key: Parameters<typeof translate>[1],
    vars?: Record<string, string | number | null | undefined>,
  ) {
    return translate(locale, key, vars);
  }

  function currentRoot() {
    return scope === "repo" && workspaceRoot.trim() ? workspaceRoot.trim() : null;
  }

  function sourceLabel(source: SourceRef) {
    if (source.kind === "github") {
      return `${source.owner}/${source.repo}`;
    }
    return source.path;
  }

  function lastPathSegment(path: string) {
    const segments = path.split("/").filter(Boolean);
    return segments[segments.length - 1] ?? path;
  }

  function localeLabel(value: Locale) {
    if (value === "ko") return tr("settings.languageKorean");
    if (value === "ja") return tr("settings.languageJapanese");
    return tr("settings.languageEnglish");
  }

  function defaultTargets(settings?: WorkspaceSettings | null): Agent[] {
    switch (settings?.target_profile) {
      case "codex-open-standard":
        return ["codex"];
      case "claude-native":
        return ["claude"];
      default:
        return ["codex", "claude"];
    }
  }

  const workspaceReady = $derived.by(() => scope === "user" || Boolean(workspaceRoot.trim()));
  const selectedStarterSource = $derived.by(() => matchStarterSource(sourceInput));
  const featuredStarterSource = $derived.by(() => STARTER_SOURCES.find((source) => source.featured) ?? STARTER_SOURCES[0]);
  const secondaryStarterSources = $derived.by(() => STARTER_SOURCES.filter((source) => source.id !== featuredStarterSource.id));
  const sourceSummary = $derived.by(() => {
    if (selectedStarterSource) {
      return {
        title: selectedStarterSource.title,
        description: tr(selectedStarterSource.descriptionKey),
        audience: tr(selectedStarterSource.audienceKey),
        badge: tr(selectedStarterSource.badgeKey),
        featured: selectedStarterSource.featured ?? false,
        url: selectedStarterSource.url,
      };
    }
    return {
      title: tr("source.customTitle"),
      description: tr("source.customCopy"),
      audience: tr("source.manualCopy"),
      badge: tr("source.badgeCustom"),
      featured: false,
      url: sourceInput,
    };
  });

  const workspaceSummary = $derived.by(() => {
    if (scope === "user") {
      return {
        title: tr("workspace.globalTitle"),
        subtitle: tr("workspace.globalSubtitle"),
        path: snapshot?.targets.codex_skills ?? tr("workspace.globalPathFallback"),
      };
    }
    return {
      title: workspaceRoot ? lastPathSegment(workspaceRoot) : tr("workspace.chooseProjectTitle"),
      subtitle: workspaceRoot
        ? tr("workspace.projectSubtitle")
        : tr("workspace.projectSubtitleEmpty"),
      path: workspaceRoot || tr("workspace.projectPathEmpty"),
    };
  });

  const activeAgentFile = $derived.by(
    () => agentFileSnapshot?.slots.find((slot) => slot.slot === activeAgentFileSlot) ?? null,
  );
  const createDrafts = $derived.by(() => snapshot?.create.drafts ?? []);
  const benchmarkSuites = $derived.by(() => snapshot?.evaluation.suites ?? []);
  const recentBenchmarkRuns = $derived.by(() => snapshot?.evaluation.recent_runs ?? []);
  const recentJobs = $derived.by(() => snapshot?.jobs.recent_jobs ?? []);
  const pendingHumanRuns = $derived.by(() =>
    recentBenchmarkRuns.filter((run) => run.status === "awaiting_human" && run.mode === "human-review"),
  );
  const activeDraftJobs = $derived.by(() =>
    selectedDraftId ? recentJobs.filter((job) => job.subject_id === selectedDraftId) : [],
  );
  const activeDraftEvidence = $derived.by(() => {
    const sourceId = createPreview?.draft.lineage.source_id;
    if (!sourceId) return null;
    return recentBenchmarkRuns.find((run) => run.candidate_source_id === sourceId) ?? null;
  });
  const activeDraftSummary = $derived.by(
    () => createDrafts.find((draft) => draft.id === selectedDraftId) ?? createDrafts[0] ?? null,
  );

  const starterDeck = $derived.by(() => {
    if (!catalog?.decks.length) return null;
    return (
      STARTER_DECK_IDS.map((id) => catalog.decks.find((deck) => deck.id === id)).find(Boolean) ??
      (catalog.decks.length === 1 ? catalog.decks[0] : null)
    );
  });

  const recommendedDecks = $derived.by(() => {
    if (!catalog) return [];
    if (starterDeck) return [starterDeck];
    return catalog.decks.slice(0, Math.min(catalog.decks.length, 2));
  });

  const filteredSkills = $derived.by(() => {
    const query = skillQuery.trim().toLowerCase();
    if (!catalog) return [];
    if (!query) return catalog.skills;
    return catalog.skills.filter((skill) => {
      const haystack = [
        skill.name,
        skill.display_name ?? "",
        skill.description,
        skill.category ?? "",
        skill.tags.join(" "),
        skill.relative_path,
      ]
        .join(" ")
        .toLowerCase();
      return haystack.includes(query);
    });
  });

  const filteredDecks = $derived.by(() => {
    const query = deckQuery.trim().toLowerCase();
    if (!catalog) return [];
    const recommendedIds = new Set(recommendedDecks.map((deck) => deck.id));
    const decks = catalog.decks.filter((deck) => !recommendedIds.has(deck.id));
    if (!query) return decks;
    return decks.filter((deck) => {
      const haystack = [deck.id, deck.name, deck.description, deck.skills.join(" ")].join(" ").toLowerCase();
      return haystack.includes(query);
    });
  });

  const filteredAgentFileTemplates = $derived.by(() => {
    const query = agentFileTemplateQuery.trim().toLowerCase();
    if (!catalog) return [];
    if (!query) return catalog.agent_file_templates;
    return catalog.agent_file_templates.filter((template) => {
      const haystack = [template.id, template.title, template.description, template.slots.join(" "), template.relative_path]
        .join(" ")
        .toLowerCase();
      return haystack.includes(query);
    });
  });

  const selectionSummary = $derived.by(() => {
    const selectedDeckSkillNames = new Set<string>();
    if (catalog) {
      for (const deckId of selectedDecks) {
        const deck = catalog.decks.find((item) => item.id === deckId);
        deck?.skills.forEach((name) => selectedDeckSkillNames.add(name));
      }
    }

    return {
      decks: selectedDecks.length,
      deckSkills: selectedDeckSkillNames.size,
      cards: selectedSkills.length,
      agentFileTemplates: selectedAgentFileTemplates.length,
      excluded: excludedSkills.length,
      targets: targets.length,
      mode: allSkills ? "all" : "selection",
    };
  });

  const hasExplicitSelection = $derived.by(() => {
    return allSkills || selectedDecks.length > 0 || selectedSkills.length > 0 || selectedAgentFileTemplates.length > 0;
  });

  const canRemoveSelection = $derived.by(() => {
    return selectedDecks.length > 0 || selectedSkills.length > 0 || selectedAgentFileTemplates.length > 0;
  });

  const installedRecord = $derived.by(() => {
    if (!snapshot || !catalog) return null;
    return snapshot.manifest.installs.find((install) => install.id === catalog.source_id) ?? null;
  });

  function lockEntry(sourceId: string | null): AppliedInstall | null {
    if (!snapshot || !sourceId) return null;
    return snapshot.lock.installs.find((install) => install.source_id === sourceId) ?? null;
  }

  const groupedPlanSkills = $derived.by(() => {
    const groups: Record<Agent, InstallPlan["skills"]> = { codex: [], claude: [] };
    if (!planState) return groups;
    for (const skill of planState.skills) {
      groups[skill.agent].push(skill);
    }
    return groups;
  });

  function installPayload(): InstallPayload {
    return {
      scope,
      root: currentRoot(),
      source: sourceInput,
      all: allSkills,
      decks: selectedDecks,
      skills: selectedSkills,
      exclude_skills: excludedSkills,
      agent_file_templates: selectedAgentFileTemplates,
      targets,
    };
  }

  function resetSelection(nextTargets: Agent[] = ["codex", "claude"]) {
    selectedDecks = [];
    selectedSkills = [];
    selectedAgentFileTemplates = [];
    excludedSkills = [];
    allSkills = false;
    targets = [...nextTargets];
    planState = null;
  }

  function clearSourceSelectionState() {
    catalog = null;
    planState = null;
    report = null;
    benchmarkResult = null;
    selectedForkSkill = "";
    deckQuery = "";
    skillQuery = "";
    agentFileTemplateQuery = "";
    showAdvancedCatalog = false;
    resetSelection(defaultTargets(snapshot?.manifest.settings));
  }

  function setSourceInput(nextValue: string) {
    if (nextValue === sourceInput) return;
    sourceInput = nextValue;
    clearSourceSelectionState();
  }

  function hydrateCreatePreview(preview: DraftPreview | null) {
    createPreview = preview;
    if (!preview) {
      draftEditorPath = "";
      draftEditorContent = "";
      return;
    }
    selectedDraftId = preview.draft.id;
    const activeDocument =
      preview.documents.find((document) => document.path === draftEditorPath) ?? preview.documents[0] ?? null;
    draftEditorPath = activeDocument?.path ?? "";
    draftEditorContent = activeDocument?.content ?? "";
  }

  function handleDraftDocumentSelect(path: string) {
    draftEditorPath = path;
    const nextDocument = createPreview?.documents.find((document) => document.path === path) ?? null;
    draftEditorContent = nextDocument?.content ?? "";
  }

  function hydrateSelectionFromInstall(sourceId: string | null) {
    if (!snapshot || !sourceId) {
      resetSelection();
      return;
    }

    const install = snapshot.manifest.installs.find((item) => item.id === sourceId);
    if (!install) {
      resetSelection(defaultTargets(snapshot.manifest.settings));
      return;
    }

    allSkills = install.selection.all;
    selectedDecks = [...install.selection.decks];
    selectedSkills = [...install.selection.skills];
    selectedAgentFileTemplates = [...install.selection.agent_file_templates];
    excludedSkills = [...install.selection.exclude_skills];
    targets = [...install.targets];
    planState = null;
  }

  async function loadWorkspace(nextScope: Scope) {
    if (nextScope === "repo" && !workspaceRoot.trim()) {
      snapshot = null;
      agentFileSnapshot = null;
      report = null;
      return;
    }

    busy = true;
    error = null;
    try {
      snapshot = await workspace(nextScope, nextScope === "repo" ? workspaceRoot.trim() : null);
      agentFileSnapshot = await agentFiles(nextScope, nextScope === "repo" ? workspaceRoot.trim() : null);
      if (!selectedDraftId && snapshot.create.drafts[0]) {
        selectedDraftId = snapshot.create.drafts[0].id;
      }
      if (catalog) {
        hydrateSelectionFromInstall(catalog.source_id);
      }
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  $effect(() => {
    scope;
    workspaceRoot;
    void loadWorkspace(scope);
  });

  $effect(() => {
    locale;
    saveLocale(locale);
  });

  async function handleInspect() {
    if (!workspaceReady) {
      error = tr("workspace.errorNeedFolder");
      return;
    }

    busy = true;
    error = null;
    report = null;
    try {
      catalog = await inspect(scope, sourceInput, currentRoot());
      selectedForkSkill = catalog.skills[0]?.name ?? "";
      hydrateSelectionFromInstall(catalog.source_id);
      deckQuery = "";
      skillQuery = "";
      agentFileTemplateQuery = "";
      activeTab = "discover";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handlePlan() {
    if (!catalog) return;
    busy = true;
    error = null;
    try {
      planState = await plan(installPayload());
      activeTab = "plan";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleInstall() {
    if (!catalog) return;
    busy = true;
    error = null;
    try {
      snapshot = await install(installPayload());
      agentFileSnapshot = await agentFiles(scope, currentRoot());
      hydrateSelectionFromInstall(catalog.source_id);
      activeTab = "library";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleRemoveAll() {
    busy = true;
    error = null;
    try {
      snapshot = await remove({
        scope,
        root: currentRoot(),
        source: sourceInput,
        decks: [],
        skills: [],
        agent_file_templates: [],
        remove_all: true,
      });
      agentFileSnapshot = await agentFiles(scope, currentRoot());
      if (catalog) {
        hydrateSelectionFromInstall(catalog.source_id);
      }
      activeTab = "library";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleSync() {
    busy = true;
    error = null;
    try {
      snapshot = await sync(scope, currentRoot());
      agentFileSnapshot = await agentFiles(scope, currentRoot());
      if (catalog) {
        hydrateSelectionFromInstall(catalog.source_id);
      }
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleUpdate() {
    busy = true;
    error = null;
    try {
      snapshot = await update(scope, currentRoot());
      agentFileSnapshot = await agentFiles(scope, currentRoot());
      if (catalog) {
        hydrateSelectionFromInstall(catalog.source_id);
      }
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleDoctor() {
    busy = true;
    error = null;
    try {
      report = await doctor(scope, currentRoot());
      activeTab = "health";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleAgentFileSave(slot: AgentFileSlot, content: string) {
    busy = true;
    error = null;
    try {
      agentFileSnapshot = await agentFilesWrite({
        scope,
        root: currentRoot(),
        slot,
        content,
      });
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleBenchmarkRun(suiteId: string) {
    busy = true;
    error = null;
    try {
      benchmarkResult = await benchmarkRun({
        scope,
        root: currentRoot(),
        suite_id: suiteId,
        source: sourceInput,
        mode: benchmarkMode,
        executor:
          benchmarkMode === "ai-judge"
            ? {
                provider: "codex-runtime",
                model: aiExecutorModel.trim() || null,
              }
            : null,
      });
      snapshot = await workspace(scope, currentRoot());
      activeTab = "benchmarks";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleCreateDraft() {
    if (!draftName.trim()) {
      error = tr("create.errorNameRequired");
      return;
    }

    busy = true;
    error = null;
    try {
      const nextPreview = await createSkillDraft({
        scope,
        root: currentRoot(),
        name: draftName.trim(),
        description: draftDescription.trim(),
        preset: draftPreset,
      });
      hydrateCreatePreview(nextPreview);
      snapshot = await workspace(scope, currentRoot());
      activeTab = "create";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handlePreviewDraft(draftId: string) {
    busy = true;
    error = null;
    try {
      const nextPreview = await previewCreateDraft({
        scope,
        root: currentRoot(),
        draft_id: draftId,
      });
      hydrateCreatePreview(nextPreview);
      activeTab = "create";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handlePromoteDraft(draftId: string) {
    busy = true;
    error = null;
    try {
      let destinationRoot: string | null = null;
      if (scope !== "repo") {
        const picked = await open({
          directory: true,
          multiple: false,
          title: tr("create.choosePromotionRoot"),
        });
        if (!picked || Array.isArray(picked)) {
          busy = false;
          return;
        }
        destinationRoot = picked;
      }

      const nextPreview = await promoteCreateDraft({
        scope,
        root: currentRoot(),
        draft_id: draftId,
        destination_root: destinationRoot,
      });
      hydrateCreatePreview(nextPreview);
      snapshot = await workspace(scope, currentRoot());
      selectedDraftId = draftId;
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleForkDraft() {
    if (!catalog || !selectedForkSkill) {
      error = tr("create.errorForkSelection");
      return;
    }

    const sourceSkill = catalog.skills.find((skill) => skill.name === selectedForkSkill);

    busy = true;
    error = null;
    try {
      const nextPreview = await forkCreateDraft({
        scope,
        root: currentRoot(),
        source: sourceInput,
        skill_name: selectedForkSkill,
        draft_name: sourceSkill?.display_name ?? sourceSkill?.name ?? selectedForkSkill,
        description: sourceSkill?.description ?? null,
      });
      hydrateCreatePreview(nextPreview);
      snapshot = await workspace(scope, currentRoot());
      activeTab = "create";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleSaveDraftDocument() {
    if (!selectedDraftId || !draftEditorPath) {
      error = tr("create.errorNoDraftDocument");
      return;
    }

    busy = true;
    error = null;
    try {
      const nextPreview = await updateCreateDraft({
        scope,
        root: currentRoot(),
        draft_id: selectedDraftId,
        relative_path: draftEditorPath,
        content: draftEditorContent,
      });
      hydrateCreatePreview(nextPreview);
      snapshot = await workspace(scope, currentRoot());
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleAugmentDraft() {
    if (!selectedDraftId || !augmentPrompt.trim()) {
      error = tr("create.errorNoDraftDocument");
      return;
    }

    busy = true;
    error = null;
    try {
      const nextPreview = await augmentCreateDraft({
        scope,
        root: currentRoot(),
        draft_id: selectedDraftId,
        prompt: augmentPrompt.trim(),
        executor: {
          provider: "codex-runtime",
          model: aiExecutorModel.trim() || null,
        },
      });
      hydrateCreatePreview(nextPreview);
      snapshot = await workspace(scope, currentRoot());
      activeTab = "create";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleSubmitHumanReview(runId: string) {
    busy = true;
    error = null;
    try {
      benchmarkResult = await submitHumanReview({
        scope,
        root: currentRoot(),
        run_id: runId,
        decision: reviewDecision,
        note: reviewNote,
      });
      snapshot = await workspace(scope, currentRoot());
      reviewNote = "";
      activeTab = "benchmarks";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleWorkJobs() {
    busy = true;
    error = null;
    try {
      await workJobs({
        scope,
        root: currentRoot(),
        session_id: "desktop",
        max_jobs: 3,
      });
      snapshot = await workspace(scope, currentRoot());
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleCancelJob(jobId: string) {
    busy = true;
    error = null;
    try {
      await cancelJob({
        scope,
        root: currentRoot(),
        job_id: jobId,
      });
      snapshot = await workspace(scope, currentRoot());
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleRetryJob(jobId: string) {
    busy = true;
    error = null;
    try {
      await retryJob({
        scope,
        root: currentRoot(),
        job_id: jobId,
      });
      snapshot = await workspace(scope, currentRoot());
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  function applyRecommendedAgentFileTemplates() {
    if (!catalog?.recipe?.recommended_agent_file_templates?.length) return;
    for (const templateId of catalog.recipe.recommended_agent_file_templates) {
      if (!selectedAgentFileTemplates.includes(templateId)) {
        selectedAgentFileTemplates = [...selectedAgentFileTemplates, templateId];
      }
    }
    planState = null;
  }

  function toggle<T>(arr: T[], value: T): T[] {
    return arr.includes(value) ? arr.filter((item) => item !== value) : [...arr, value];
  }

  function dedup(items: string[]): string[] {
    return [...new Set(items)].sort();
  }

  function applyBestFitSelection() {
    if (!catalog) return;

    if (!allSkills && !selectedDecks.length && !selectedSkills.length) {
      if (starterDeck) {
        selectedDecks = [starterDeck.id];
      } else if (catalog.skills.length > 0 && catalog.skills.length <= 3) {
        allSkills = true;
      }
    }

    if (catalog.recipe?.recommended_agent_file_templates?.length) {
      selectedAgentFileTemplates = dedup([
        ...selectedAgentFileTemplates,
        ...catalog.recipe.recommended_agent_file_templates,
      ]);
    }

    planState = null;
  }

  function toggleDeck(id: string) {
    selectedDecks = toggle(selectedDecks, id);
    planState = null;
  }

  function toggleSkill(skill: SkillInfo) {
    selectedSkills = toggle(selectedSkills, skill.name);
    planState = null;
  }

  function toggleAgentFileTemplate(template: AgentFileTemplate) {
    selectedAgentFileTemplates = toggle(selectedAgentFileTemplates, template.id);
    planState = null;
  }

  function toggleExclude(skill: SkillInfo) {
    excludedSkills = toggle(excludedSkills, skill.name);
    planState = null;
  }

  function toggleTarget(agent: Agent) {
    targets = toggle(targets, agent);
    planState = null;
  }

  async function handleQuickApply() {
    if (!catalog) return;
    applyBestFitSelection();

    busy = true;
    error = null;
    try {
      const nextPlan = await plan(installPayload());
      planState = nextPlan;
      if (nextPlan.conflicts.length) {
        activeTab = "plan";
        return;
      }
      snapshot = await install(installPayload());
      agentFileSnapshot = await agentFiles(scope, currentRoot());
      hydrateSelectionFromInstall(catalog.source_id);
      activeTab = "library";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function inspectSourceChoice(source: string) {
    setSourceInput(source);
    busy = true;
    error = null;
    report = null;
    try {
      catalog = await inspect(scope, source, currentRoot());
      sourceInput = source;
      selectedForkSkill = catalog.skills[0]?.name ?? "";
      hydrateSelectionFromInstall(catalog.source_id);
      deckQuery = "";
      skillQuery = "";
      agentFileTemplateQuery = "";
      activeTab = "discover";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleRepoSelect() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: tr("workspace.chooseProjectTitle"),
      });
      if (selected !== null && !Array.isArray(selected)) {
        if (scope === "repo" && workspaceRoot === selected) {
          void loadWorkspace("repo");
        } else {
          workspaceRoot = selected;
          scope = "repo";
        }
      }
    } catch (err) {
      error = String(err);
    }
  }

  function handleUserWorkspace() {
    if (scope === "user" && !workspaceRoot) {
      void loadWorkspace("user");
      return;
    }
    workspaceRoot = "";
    scope = "user";
  }

  async function handleRemoveSelection() {
    if (!catalog || !canRemoveSelection) return;
    busy = true;
    error = null;
    try {
      snapshot = await remove({
        scope,
        root: currentRoot(),
        source: sourceInput,
        decks: selectedDecks,
        skills: selectedSkills,
        agent_file_templates: selectedAgentFileTemplates,
        remove_all: false,
      });
      agentFileSnapshot = await agentFiles(scope, currentRoot());
      hydrateSelectionFromInstall(catalog.source_id);
      activeTab = "library";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }
</script>

<div class="app-shell">
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark">P</div>
      <div>
        <div class="brand-title">praxis</div>
        <div class="brand-subtitle">{tr("brand.subtitle")}</div>
      </div>
    </div>

    <nav class="sidebar-nav">
      <button class="nav-item" class:active={activeTab === "discover"} onclick={() => (activeTab = "discover")}>
        <Search size={18} strokeWidth={2.5} /> {tr("nav.catalog")}
      </button>
      <button class="nav-item" class:active={activeTab === "library"} onclick={() => (activeTab = "library")}>
        <CheckCircle size={18} strokeWidth={2.5} /> {tr("nav.installed")}
      </button>
      <button class="nav-item" class:active={activeTab === "create"} onclick={() => (activeTab = "create")}>
        <DownloadCloud size={18} strokeWidth={2.5} /> {tr("nav.create")}
      </button>
      <button class="nav-item" class:active={activeTab === "benchmarks"} onclick={() => (activeTab = "benchmarks")}>
        <BookOpen size={18} strokeWidth={2.5} /> {tr("nav.benchmarks")}
      </button>
      <button class="nav-item" class:active={activeTab === "connections"} onclick={() => (activeTab = "connections")}>
        <ArrowRight size={18} strokeWidth={2.5} /> {tr("nav.connections")}
      </button>
      <button class="nav-item" class:active={activeTab === "health"} onclick={() => (activeTab = "health")}>
        <Activity size={18} strokeWidth={2.5} /> {tr("nav.doctor")}
      </button>
      <button class="nav-item" class:active={activeTab === "settings"} onclick={() => (activeTab = "settings")}>
        <Settings2 size={18} strokeWidth={2.5} /> {tr("nav.settings")}
      </button>
    </nav>

    <section class="panel">
      <div class="field-label">{tr("workspace.choose")}</div>
      <button class:active={scope === "user"} class="workspace-tile" onclick={handleUserWorkspace}>
        <User size={18} />
        <div>
          <strong>{tr("workspace.globalLibrary")}</strong>
          <span>{tr("workspace.globalLibraryHint")}</span>
        </div>
      </button>
      <button class:active={scope === "repo"} class="workspace-tile" onclick={handleRepoSelect}>
        <Folder size={18} />
        <div>
          <strong>{tr("workspace.projectFolder")}</strong>
          <span>{scope === "repo" && workspaceRoot ? workspaceRoot : tr("workspace.projectFolderHint")}</span>
        </div>
      </button>
      <div class="workspace-chip-row">
        <span class="workspace-chip">{workspaceSummary.title}</span>
      </div>
      <div class="workspace-path">{workspaceSummary.path}</div>
      <div class="workspace-copy">{workspaceSummary.subtitle}</div>
    </section>

    <section class="panel">
      <div class="field-label">{tr("source.label")}</div>
      <div class="panel-headline">
        <strong>{tr("source.currentChoice")}</strong>
        <span>{tr("source.sidebarCopy")}</span>
      </div>
      <StarterSourceCard
        source={{
          id: selectedStarterSource?.id ?? "custom-source",
          title: sourceSummary.title,
          url: sourceSummary.url,
          descriptionKey: "source.customCopy",
          audienceKey: "source.manualCopy",
          badgeKey: "source.badgeCustom",
          featured: sourceSummary.featured,
        }}
        selected={true}
        compact={true}
        badge={sourceSummary.badge}
        featuredLabel={tr("source.badgeFeatured")}
        description={sourceSummary.description}
        audience={sourceSummary.audience}
        actionLabel={tr("source.inspectAction")}
        actionStateLabel={tr("source.readyActionState")}
        staticCard={true}
      />
      <details class="inline-details">
        <summary>{tr("source.manualTitle")}</summary>
        <div class="inline-details-copy">{tr("source.manualCopy")}</div>
        <textarea
          class="source-input"
          rows="2"
          value={sourceInput}
          placeholder={tr("source.placeholder")}
          oninput={(event) => setSourceInput((event.currentTarget as HTMLTextAreaElement).value)}
        ></textarea>
      </details>
      <div class="inline-note">{tr("source.changeHint")}</div>
      <button class="primary wide" disabled={busy || !workspaceReady} onclick={handleInspect}>
        {#if busy}
          <RefreshCw size={16} class="spin" /> {tr("source.inspecting")}
        {:else}
          <Search size={16} /> {tr("source.inspect")}
        {/if}
      </button>
      {#if catalog}
        <button class="wide" style="margin-top: 8px;" disabled={busy} onclick={handleQuickApply}>
          <CheckCircle size={14} /> {tr("common.quickApply")} {tr("common.useBestFit")}
        </button>
      {/if}
      {#if !workspaceReady}
        <div class="inline-note">{tr("workspace.requireFolder")}</div>
      {/if}
    </section>

  </aside>

  <main class="main">
    <header class="hero">
      <h1>
        {#if activeTab === "discover"}{tr("hero.catalog.title")}
        {:else if activeTab === "plan"}{tr("hero.plan.title")}
        {:else if activeTab === "library"}{tr("hero.installed.title")}
        {:else if activeTab === "agent-files"}{tr("hero.agentFiles.title")}
        {:else if activeTab === "create"}{tr("hero.create.title")}
        {:else if activeTab === "benchmarks"}{tr("hero.benchmarks.title")}
        {:else if activeTab === "connections"}{tr("hero.connections.title")}
        {:else if activeTab === "health"}{tr("hero.doctor.title")}
        {:else if activeTab === "settings"}{tr("hero.settings.title")}
        {/if}
      </h1>
      <p>
        {#if activeTab === "discover"}{tr("hero.catalog.copy")}
        {:else if activeTab === "plan"}{tr("hero.plan.copy")}
        {:else if activeTab === "library"}{tr("hero.installed.copy")}
        {:else if activeTab === "agent-files"}{tr("hero.agentFiles.copy")}
        {:else if activeTab === "create"}{tr("hero.create.copy")}
        {:else if activeTab === "benchmarks"}{tr("hero.benchmarks.copy")}
        {:else if activeTab === "connections"}{tr("hero.connections.copy")}
        {:else if activeTab === "health"}{tr("hero.doctor.copy")}
        {:else if activeTab === "settings"}{tr("hero.settings.copy")}
        {/if}
      </p>
    </header>

    {#if error}
      <div class="alert error" transition:slide>
        <strong>{tr("common.errorPrefix")}</strong> {error}
      </div>
    {/if}

    <div class="view-container">
      {#if activeTab === "discover"}
        <div in:fade={{ duration: 200, delay: 100 }}>
          {#if catalog}
            <section class="section">
              <div class="workspace-banner">
                <div>
                  <div class="eyebrow">{tr("workspace.current")}</div>
                  <h2>{workspaceSummary.title}</h2>
                  <p>{workspaceSummary.path}</p>
                </div>
                <div>
                  <div class="eyebrow">{tr("source.labelCurrent")}</div>
                  <h2>{catalog.label}</h2>
                  <p>{sourceLabel(catalog.source)}</p>
                </div>
              </div>
            </section>

            <section class="section">
              <div class="section-head">
                <div>
                  <h2>{tr("catalog.recommendedTitle")}</h2>
                  <div class="meta">{tr("catalog.recommendedCopy")}</div>
                </div>
                <div class="button-row compact">
                  <button disabled={busy || !catalog} onclick={applyBestFitSelection}>{tr("common.useBestFit")}</button>
                  <button class="primary" disabled={busy || !catalog} onclick={handleQuickApply}>
                    {tr("common.quickApply")} <ArrowRight size={16} />
                  </button>
                  <button disabled={busy || !catalog} onclick={handlePlan}>{tr("common.previewPlan")}</button>
                </div>
              </div>

              {#if recommendedDecks.length}
                <div class="deck-grid">
                  {#each recommendedDecks as deck}
                    <DeckCard
                      {deck}
                      recommended={true}
                      recommendedLabel={tr("deck.recommended")}
                      deckLabel={tr("deck.label")}
                      synthesizedLabel={tr("deck.synthesized")}
                      declaredLabel={tr("deck.declared")}
                      cardsInsideLabel={tr("deck.cardsInside")}
                      moreLabel={(count) => tr("deck.more", { count })}
                      selected={selectedDecks.includes(deck.id)}
                      onclick={() => toggleDeck(deck.id)}
                    />
                  {/each}
                </div>
              {:else}
                <div class="empty-card">{tr("catalog.emptyNoDeck")}</div>
              {/if}

              <div class="summary-grid compact-grid">
                <div class="summary-box"><span>{tr("catalog.summaryDecks")}</span><strong>{selectionSummary.decks}</strong></div>
                <div class="summary-box"><span>{tr("catalog.summaryDeckCards")}</span><strong>{selectionSummary.deckSkills}</strong></div>
                <div class="summary-box"><span>{tr("catalog.summaryManualCards")}</span><strong>{selectionSummary.cards}</strong></div>
                <div class="summary-box"><span>{tr("catalog.summaryGuideSlots")}</span><strong>{selectionSummary.agentFileTemplates}</strong></div>
              </div>

              {#if catalog.recipe || catalog.warnings.length || installedRecord}
                <div class="stacked-notes">
                  {#if catalog.recipe}
                    <div class="alert note">
                      <div>
                        <strong>{catalog.recipe.label}</strong> · {catalog.recipe.description}
                      </div>
                      {#if catalog.recipe.recommended_agent_file_templates.length}
                        <button onclick={applyRecommendedAgentFileTemplates}>{tr("common.useRecommendedAgentFiles")}</button>
                      {/if}
                    </div>
                  {/if}
                  {#if installedRecord}
                    <div class="alert note">
                      {tr("catalog.recipeInstalled", {
                        decks: installedRecord.selection.decks.length || 0,
                        skills: lockEntry(installedRecord.id)?.skills.length ?? 0,
                      })}
                    </div>
                  {/if}
                  {#if catalog.warnings.length}
                    <div class="alert warn">
                      {#each catalog.warnings as warning}<div>{warning}</div>{/each}
                    </div>
                  {/if}
                </div>
              {/if}
            </section>

            <details class="panel advanced-panel" bind:open={showAdvancedCatalog}>
              <summary>{tr("common.advancedControls")}</summary>
              <div class="advanced-copy">
                {tr("catalog.advancedCopy")}
              </div>

              <div class="advanced-targets">
                <label class="check">
                  <input type="checkbox" checked={targets.includes("codex")} onchange={() => toggleTarget("codex")} />
                  Codex
                </label>
                <label class="check">
                  <input type="checkbox" checked={targets.includes("claude")} onchange={() => toggleTarget("claude")} />
                  Claude
                </label>
                <label class="check">
                  <input type="checkbox" bind:checked={allSkills} onchange={() => (planState = null)} />
                  {tr("catalog.useAllSkills")}
                </label>
              </div>

              <section class="section">
                <div class="section-head">
                  <h2>{tr("catalog.moreDecks")}</h2>
                  <div class="meta">{filteredDecks.length}</div>
                </div>
                <input class="search-input" bind:value={deckQuery} placeholder={tr("catalog.searchDecks")} />
                <div class="deck-grid compact-decks">
                  {#each filteredDecks as deck}
                    <DeckCard
                      {deck}
                      recommendedLabel={tr("deck.recommended")}
                      deckLabel={tr("deck.label")}
                      synthesizedLabel={tr("deck.synthesized")}
                      declaredLabel={tr("deck.declared")}
                      cardsInsideLabel={tr("deck.cardsInside")}
                      moreLabel={(count) => tr("deck.more", { count })}
                      selected={selectedDecks.includes(deck.id)}
                      onclick={() => toggleDeck(deck.id)}
                    />
                  {/each}
                </div>
              </section>

              <section class="section">
                <div class="section-head">
                  <h2>{tr("catalog.skillCards")}</h2>
                  <div class="meta">{filteredSkills.length}</div>
                </div>
                <input class="search-input" bind:value={skillQuery} placeholder={tr("catalog.searchSkills")} />
                <div class="grid">
                  {#each filteredSkills as skill}
                    <div class="stack-card skill-card-wrapper">
                      <Card
                        eyebrow={tr("common.skill")}
                        title={skill.display_name ?? skill.name}
                        description={skill.description}
                        badge={skill.category ?? skill.root_component}
                        secondary={skill.relative_path}
                        selected={selectedSkills.includes(skill.name)}
                        onclick={() => toggleSkill(skill)}
                      />
                      <button
                        class:active={excludedSkills.includes(skill.name)}
                        class="tiny-toggle"
                        onclick={() => toggleExclude(skill)}
                      >
                        {excludedSkills.includes(skill.name) ? tr("catalog.excluded") : tr("catalog.exclude")}
                      </button>
                    </div>
                  {/each}
                </div>
              </section>

              <section class="section">
                <div class="section-head">
                  <h2>{tr("catalog.agentFileTemplates")}</h2>
                  <div class="meta">{filteredAgentFileTemplates.length}</div>
                </div>
                <input class="search-input" bind:value={agentFileTemplateQuery} placeholder={tr("catalog.searchAgentFileTemplates")} />
                <div class="guide-slot-list">
                  {#each filteredAgentFileTemplates as template}
                    <button
                      class:selected={selectedAgentFileTemplates.includes(template.id)}
                      class="guide-slot"
                      onclick={() => toggleAgentFileTemplate(template)}
                    >
                      <div>
                        <div class="eyebrow">{template.slots.join(", ")}</div>
                        <strong>{template.title}</strong>
                        <p>{template.description}</p>
                      </div>
                      <div class="guide-slot-meta">
                        <span>{template.relative_path}</span>
                        <span>{selectedAgentFileTemplates.includes(template.id) ? tr("common.selected") : tr("common.optional")}</span>
                      </div>
                    </button>
                  {/each}
                </div>
              </section>
            </details>
          {:else}
            <section class="section">
              <div class="launchpad">
                <div class="launchpad-stage">
                  <div class="launchpad-intro">
                    <div>
                      <div class="eyebrow">{tr("workspace.current")}</div>
                      <h2>{tr("catalog.emptyInspectTitle")}</h2>
                      <p>{tr("catalog.emptyInspectCopy")}</p>
                    </div>
                    <div class="button-row compact">
                      <button class:primary={scope === "user"} onclick={handleUserWorkspace}>
                        {tr("workspace.globalLibrary")}
                      </button>
                      <button class:primary={scope === "repo"} onclick={handleRepoSelect}>
                        {tr("workspace.projectFolder")}
                      </button>
                      <button class="primary" disabled={busy || !workspaceReady} onclick={handleInspect}>
                        <Search size={16} /> {tr("source.inspect")}
                      </button>
                    </div>
                  </div>

                  <div class="launch-rhythm">
                    <div class="rhythm-step active">
                      <span>01</span>
                      <strong>{tr("workspace.focus")}</strong>
                      <p>{workspaceSummary.title}</p>
                    </div>
                    <div class="rhythm-step active">
                      <span>02</span>
                      <strong>{tr("source.currentChoice")}</strong>
                      <p>{sourceSummary.title}</p>
                    </div>
                    <div class="rhythm-step">
                      <span>03</span>
                      <strong>{tr("catalog.recommendedTitle")}</strong>
                      <p>{tr("source.afterInspectCopy")}</p>
                    </div>
                  </div>
                </div>

                <div class="launch-grid">
                  <div class="launch-step">
                    <div class="launch-step-number">1</div>
                    <div>
                      <h3>{tr("workspace.focus")}</h3>
                      <p>{workspaceSummary.subtitle}</p>
                      <div class="workspace-pill-row">
                        <span class="workspace-chip">{workspaceSummary.title}</span>
                        {#if scope === "repo" && workspaceRoot}
                          <span class="workspace-chip subtle">{tr("workspace.projectFolder")}</span>
                        {/if}
                      </div>
                    </div>
                  </div>

                  <div class="launch-step">
                    <div class="launch-step-number">2</div>
                    <div>
                      <h3>{tr("source.starterTitle")}</h3>
                      <p>{tr("source.startChoiceCopy")}</p>
                    </div>
                  </div>
                </div>

                <div class="starter-showcase">
                  <div class="starter-showcase-copy">
                    <div class="eyebrow">{tr("source.badgeFeatured")}</div>
                    <h3>{tr("source.featuredTitle")}</h3>
                    <p>{tr("source.featuredCopy")}</p>
                  </div>
                  <StarterSourceCard
                    source={featuredStarterSource}
                    selected={selectedStarterSource?.id === featuredStarterSource.id}
                    badge={tr(featuredStarterSource.badgeKey)}
                    featuredLabel={tr("source.badgeFeatured")}
                    description={tr(featuredStarterSource.descriptionKey)}
                    audience={tr(featuredStarterSource.audienceKey)}
                    actionLabel={tr("source.inspectAction")}
                    actionStateLabel={tr("source.openActionState")}
                    onclick={() => void inspectSourceChoice(featuredStarterSource.url)}
                  />
                </div>

                <div class="section-head">
                  <div>
                    <h2>{tr("source.libraryTitle")}</h2>
                    <div class="meta">{tr("source.libraryCopy")}</div>
                  </div>
                  <div class="meta">{STARTER_SOURCES.length}</div>
                </div>

                <div class="starter-grid">
                  {#each secondaryStarterSources as starter}
                    <StarterSourceCard
                      source={starter}
                      selected={selectedStarterSource?.id === starter.id}
                      badge={tr(starter.badgeKey)}
                      featuredLabel={tr("source.badgeFeatured")}
                      description={tr(starter.descriptionKey)}
                      audience={tr(starter.audienceKey)}
                      actionLabel={tr("source.inspectAction")}
                      actionStateLabel={tr("source.openActionState")}
                      onclick={() => void inspectSourceChoice(starter.url)}
                    />
                  {/each}
                </div>
              </div>
            </section>
          {/if}
        </div>
      {/if}

      {#if activeTab === "plan"}
        <div in:fade={{ duration: 200, delay: 100 }}>
          {#if planState}
            <div class="alert note" style="align-items: center; justify-content: space-between;">
              <div>
                {tr("plan.receiptIntro", { label: planState.label })}
              </div>
              <button class="primary" disabled={busy || Boolean(planState.conflicts.length)} onclick={handleInstall}>
                <DownloadCloud size={16} /> {tr("plan.applySelection")}
              </button>
            </div>

            <section class="section">
              <div class="summary-grid">
                <div class="summary-box"><span>{tr("plan.totalSkills")}</span><strong>{planState.summary.total_skills}</strong></div>
                <div class="summary-box"><span>{tr("plan.codexSkills")}</span><strong>{planState.summary.codex_skills}</strong></div>
                <div class="summary-box"><span>{tr("plan.claudeSkills")}</span><strong>{planState.summary.claude_skills}</strong></div>
                <div class="summary-box"><span>{tr("plan.agentFiles")}</span><strong>{planState.summary.total_agent_file_actions}</strong></div>
              </div>

              <div class="receipt-panel">
                <div class="receipt-row"><span>{tr("plan.workspace")}</span><strong>{workspaceSummary.path}</strong></div>
                <div class="receipt-row"><span>{tr("plan.source")}</span><strong>{planState.label}</strong></div>
                <div class="receipt-row"><span>{tr("plan.targets")}</span><strong>{planState.targets.join(", ")}</strong></div>
                <div class="receipt-row"><span>{tr("plan.decks")}</span><strong>{planState.selection.decks.join(", ") || tr("common.none")}</strong></div>
                <div class="receipt-row"><span>{tr("plan.skillCards")}</span><strong>{planState.selection.skills.join(", ") || tr("common.none")}</strong></div>
                <div class="receipt-row"><span>{tr("plan.agentFileTemplates")}</span><strong>{planState.selection.agent_file_templates.join(", ") || tr("common.none")}</strong></div>
                <div class="receipt-row"><span>{tr("plan.excluded")}</span><strong>{planState.selection.exclude_skills.join(", ") || tr("common.none")}</strong></div>
              </div>
            </section>

            {#if planState.conflicts.length}
              <div class="alert error" transition:slide>
                <strong>{tr("plan.conflictsDetected")}</strong>
                <ul style="margin: 8px 0 0 -20px;">
                  {#each planState.conflicts as conflict}<li>{conflict}</li>{/each}
                </ul>
              </div>
            {/if}

            {#if planState.warnings.length}
              <div class="alert warn" transition:slide>
                <ul style="margin: 0 0 0 -20px;">
                  {#each planState.warnings as warning}<li>{warning}</li>{/each}
                </ul>
              </div>
            {/if}

            {#if planState.notes.length}
              <div class="alert note" transition:slide>
                <ul style="margin: 0 0 0 -20px;">
                  {#each planState.notes as note}<li>{note}</li>{/each}
                </ul>
              </div>
            {/if}

            <section class="section">
              <div class="section-head">
                <h2>{tr("plan.plannedSkillWrites")}</h2>
                <div class="meta">{planState.skills.length}</div>
              </div>
              <div class="receipt-groups">
                {#each Object.entries(groupedPlanSkills) as [agent, skills]}
                  <div class="receipt-group">
                    <div class="receipt-group-head">
                      <h3>{agent}</h3>
                      <span>{skills.length}</span>
                    </div>
                    {#if skills.length}
                      <div class="grid plan-skill-grid">
                        {#each skills as skill}
                          <Card
                            eyebrow={agent}
                            title={skill.display_name ?? skill.name}
                            description={skill.source_relative_path}
                            badge={skill.category}
                            secondary={skill.target_path}
                            footerTag={tr("common.skill")}
                            staticCard={true}
                          />
                        {/each}
                      </div>
                    {:else}
                      <div class="receipt-empty">{tr("plan.noWritesForTarget")}</div>
                    {/if}
                  </div>
                {/each}
              </div>
            </section>

            {#if planState.agent_file_actions.length}
              <section class="section">
                <div class="section-head">
                  <h2>{tr("plan.agentFileChanges")}</h2>
                  <div class="meta">{planState.agent_file_actions.length}</div>
                </div>
                <div class="receipt-panel">
                  {#each planState.agent_file_actions as action}
                    <div class="receipt-row stacked">
                      <div>
                        <span>{action.slot}</span>
                        <strong>{action.title}</strong>
                      </div>
                      <strong>{action.target_path}</strong>
                    </div>
                  {/each}
                </div>
              </section>
            {/if}

            {#if planState.bundles.length}
              <section class="section">
                <div class="section-head">
                  <h2>{tr("plan.companionBundles")}</h2>
                  <div class="meta">{planState.bundles.length}</div>
                </div>
                <div class="receipt-panel">
                  {#each planState.bundles as bundle}
                    <div class="receipt-row stacked">
                      <div>
                        <span>{bundle.agent}</span>
                        <strong>{bundle.id}</strong>
                      </div>
                      <strong>{bundle.target_path}</strong>
                    </div>
                  {/each}
                </div>
              </section>
            {/if}
          {:else}
            <section class="section empty">
              <Layers size={48} color="rgba(255,255,255,0.2)" style="margin-bottom: 16px;" />
              <h2>{tr("plan.emptyTitle")}</h2>
              <p>{tr("plan.emptyCopy")}</p>
              <button onclick={() => (activeTab = "discover")} style="margin-top: 16px;">{tr("plan.backToCatalog")}</button>
            </section>
          {/if}
        </div>
      {/if}

      {#if activeTab === "library"}
        <div in:fade={{ duration: 200, delay: 100 }}>
          <section class="section">
            <div class="section-head">
              <div>
                <h2>{tr("installed.sourceShelves")}</h2>
                <div class="meta">{tr("installed.sourceShelvesCopy")}</div>
              </div>
              <div class="meta">{snapshot?.manifest.installs.length ?? 0}</div>
            </div>
            {#if snapshot?.manifest.installs.length}
              <div class="list">
                {#each snapshot.manifest.installs as install}
                  <InstalledSourceCard
                    {install}
                    applied={lockEntry(install.id)}
                    current={catalog?.source_id === install.id}
                    labels={{
                      currentSource: tr("installed.currentSource"),
                      emptySelection: tr("installed.emptySelection"),
                      appliedSkills: tr("installed.appliedSkills"),
                      agentFileActions: tr("installed.agentFileActions"),
                      bundles: tr("installed.bundles"),
                      reference: tr("installed.reference"),
                      local: tr("common.local"),
                      selectionAll: tr("installed.selectionAll"),
                      selectionDecks: (count) => tr("installed.selectionDecks", { count }),
                      selectionCards: (count) => tr("installed.selectionCards", { count }),
                      selectionAgentFileTemplates: (count) => tr("installed.selectionAgentFiles", { count }),
                      metaExcluded: (items) => tr("installed.metaExcluded", { items }),
                      metaAgentFileTemplates: (items) => tr("installed.metaAgentFiles", { items }),
                      metaSourceHash: (hash) => tr("installed.metaSourceHash", { hash }),
                    }}
                  />
                {/each}
              </div>
              <div class="button-row" style="margin-top: 24px; justify-content: flex-end;">
                <button onclick={() => (showAdvancedWorkspace = !showAdvancedWorkspace)}>
                  {showAdvancedWorkspace ? tr("common.hide") : tr("common.show")} {tr("common.advancedActions")}
                </button>
                <button onclick={handleRemoveSelection} disabled={busy || !catalog || !canRemoveSelection}>
                  <Trash2 size={16} /> {tr("installed.removeCurrentSelection")}
                </button>
                <button
                  onclick={handleRemoveAll}
                  disabled={busy || !catalog}
                  class="error-btn"
                  style="color:var(--error); border-color:rgba(255,69,58,0.4);"
                >
                  <Trash2 size={16} /> {tr("installed.removeSourceCatalog")}
                </button>
              </div>
            {:else}
              <div class="empty-card" style="text-align: center;">{tr("installed.empty")}</div>
            {/if}
          </section>

          {#if snapshot}
            <section class="section">
              <div class="section-head">
                <h2>{tr("installed.physicalTargets")}</h2>
                <div class="meta">{tr("installed.physicalTargetsCopy")}</div>
              </div>
              <div class="receipt-panel">
                <div class="receipt-row"><span>{tr("installed.targetCodexSkills")}</span><strong>{snapshot.targets.codex_skills}</strong></div>
                <div class="receipt-row"><span>{tr("installed.targetClaudeSkills")}</span><strong>{snapshot.targets.claude_skills}</strong></div>
                <div class="receipt-row"><span>{tr("installed.targetAgents")}</span><strong>{snapshot.targets.codex_agents}</strong></div>
                <div class="receipt-row"><span>{tr("installed.targetAgentsOverride")}</span><strong>{snapshot.targets.codex_override}</strong></div>
                <div class="receipt-row"><span>{tr("installed.targetAgentAlias")}</span><strong>{snapshot.targets.codex_agent_alias}</strong></div>
                <div class="receipt-row"><span>{tr("installed.targetClaudeRoot")}</span><strong>{snapshot.targets.claude_root}</strong></div>
                <div class="receipt-row"><span>{tr("installed.targetClaudeDot")}</span><strong>{snapshot.targets.claude_dot}</strong></div>
              </div>
            </section>
          {/if}

          {#if showAdvancedWorkspace}
            <section class="section">
              <div class="section-head">
                <h2>{tr("common.advancedActions")}</h2>
                <div class="meta">{tr("installed.advancedCopy")}</div>
              </div>
              <div class="button-row">
                <button disabled={busy} onclick={handleSync}><RefreshCw size={14} /> {tr("common.sync")}</button>
                <button disabled={busy} onclick={handleUpdate}><RefreshCw size={14} /> {tr("common.update")}</button>
                <button disabled={busy} onclick={handleDoctor}><Activity size={14} /> {tr("common.doctor")}</button>
                <button disabled={busy} onclick={() => (activeTab = "agent-files")}><BookOpen size={14} /> {tr("common.agentFiles")}</button>
              </div>
            </section>
          {/if}
        </div>
      {/if}

      {#if activeTab === "agent-files"}
        <div in:fade={{ duration: 200, delay: 100 }}>
          <section class="section">
            <div class="guide-slot-list" style="margin-bottom: 24px;">
              {#each AGENT_FILE_SLOTS as slot}
                <button class:selected={activeAgentFileSlot === slot} class="guide-slot" onclick={() => (activeAgentFileSlot = slot)}>
                  <div>
                    <div class="eyebrow">{tr("agentFiles.slot")}</div>
                    <strong>{slot}</strong>
                    <p>{agentFileSnapshot?.slots.find((item) => item.slot === slot)?.target_path ?? tr("agentFiles.noPath")}</p>
                  </div>
                  <div class="guide-slot-meta">
                    <span>{activeAgentFileSlot === slot ? tr("common.editing") : tr("common.open")}</span>
                  </div>
                </button>
              {/each}
            </div>
            <AgentFileEditor
              slotState={activeAgentFile}
              {busy}
              onSave={handleAgentFileSave}
              labels={{
                save: tr("agentFileEditor.save"),
                managedBlocks: (count) => tr("agentFileEditor.managedBlocks", { count }),
                exists: tr("agentFileEditor.exists"),
                notCreated: tr("agentFileEditor.notCreated"),
                emptyTitle: tr("agentFileEditor.emptyTitle"),
                emptyCopy: tr("agentFileEditor.emptyCopy"),
              }}
            />
          </section>
        </div>
      {/if}

      {#if activeTab === "create"}
        <div in:fade={{ duration: 200, delay: 100 }}>
          <section class="section">
            <div class="section-heading">
              <div>
                <h2>{tr("create.draftsTitle")}</h2>
                <p>{tr("create.draftsCopy")}</p>
              </div>
            </div>

            <div class="panel stack-panel" style="margin-bottom: 20px;">
              <div class="field-grid">
                <label class="stack-field">
                  <span class="field-label">{tr("create.nameLabel")}</span>
                  <input class="source-input" bind:value={draftName} placeholder={tr("create.namePlaceholder")} />
                </label>
                <label class="stack-field">
                  <span class="field-label">{tr("create.presetLabel")}</span>
                  <select class="source-input" bind:value={draftPreset}>
                    <option value="skill">skill</option>
                  </select>
                </label>
              </div>
              <label class="stack-field">
                <span class="field-label">{tr("create.descriptionLabel")}</span>
                <textarea
                  class="source-input"
                  rows="4"
                  bind:value={draftDescription}
                  placeholder={tr("create.descriptionPlaceholder")}
                ></textarea>
              </label>
              <label class="stack-field">
                <span class="field-label">{tr("create.augmentPromptLabel")}</span>
                <textarea class="source-input" rows="3" bind:value={augmentPrompt}></textarea>
              </label>
              <div class="button-row">
                <button class="primary" disabled={busy} onclick={handleCreateDraft}>
                  <DownloadCloud size={16} /> {tr("create.createDraft")}
                </button>
                {#if catalog?.skills.length}
                  <label class="stack-field inline-select">
                    <span class="field-label">{tr("create.forkSkillLabel")}</span>
                    <select class="source-input" bind:value={selectedForkSkill}>
                      {#each catalog.skills as skill}
                        <option value={skill.name}>{skill.display_name ?? skill.name}</option>
                      {/each}
                    </select>
                  </label>
                  <button disabled={busy} onclick={handleForkDraft}>
                    <Layers size={16} /> {tr("create.forkAction")}
                  </button>
                {/if}
                {#if activeDraftSummary}
                  <button disabled={busy} onclick={() => handlePreviewDraft(activeDraftSummary.id)}>
                    {tr("create.refreshPreview")}
                  </button>
                  <button disabled={busy} onclick={handleAugmentDraft}>
                    <Sparkles size={16} /> {tr("create.augmentAction")}
                  </button>
                {/if}
              </div>
            </div>

            {#if createDrafts.length}
              <div class="list">
                {#each createDrafts as draft}
                  <div class:selected-row={selectedDraftId === draft.id} class="check-row info interactive-row">
                    <strong>{draft.artifact_kind}</strong>
                    <span>{draft.lineage.origin_kind}</span>
                    <p>{draft.name} · {draft.version_id}</p>
                    <p>
                      {draft.lineage.parent_name ?? tr("create.lineageRoot")} ·
                      {draft.lineage.parent_version_id ?? tr("create.lineageStandalone")}
                    </p>
                    <div class="row-actions">
                      <p>{tr("create.updatedAt", { value: draft.updated_at })}</p>
                      <div class="button-row compact">
                        <button disabled={busy} onclick={() => handlePreviewDraft(draft.id)}>{tr("create.previewAction")}</button>
                        <button class="primary" disabled={busy} onclick={() => handlePromoteDraft(draft.id)}>
                          {tr("create.promoteAction")}
                        </button>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="empty-inline">
                <p>{tr("create.emptyCopy")}</p>
              </div>
            {/if}

            {#if createPreview}
              <div class="panel stack-panel" style="margin-top: 20px;">
                <div class="section-heading">
                  <div>
                    <h2>{tr("create.previewTitle")}</h2>
                    <p>{tr("create.previewCopy")}</p>
                  </div>
                </div>
                <div class="stat">
                  <span>{tr("create.previewDraft")}</span>
                  <strong>{createPreview.draft.name}</strong>
                </div>
                <div class="stat">
                  <span>{tr("create.promotionTarget")}</span>
                  <strong>{createPreview.promotion_target}</strong>
                </div>
                <div class="receipt-panel">
                  <div class="receipt-row">
                    <span>{tr("create.lineageOrigin")}</span>
                    <strong>{createPreview.draft.lineage.origin_kind}</strong>
                  </div>
                  <div class="receipt-row">
                    <span>{tr("create.lineageParent")}</span>
                    <strong>{createPreview.draft.lineage.parent_name ?? tr("create.lineageRoot")}</strong>
                  </div>
                  <div class="receipt-row">
                    <span>{tr("create.reviewChangedFiles")}</span>
                    <strong>{createPreview.review.changed_files}</strong>
                  </div>
                  <div class="receipt-row">
                    <span>{tr("create.reviewPendingJobs")}</span>
                    <strong>{activeDraftJobs.length || createPreview.review.pending_job_count}</strong>
                  </div>
                  <div class="receipt-row">
                    <span>{tr("create.reviewLatestEvidence")}</span>
                    <strong>{activeDraftEvidence?.recommendation ?? createPreview.review.latest_recommendation ?? tr("common.none")}</strong>
                  </div>
                </div>
                {#if createPreview.documents.length}
                  <div class="field-grid">
                    <label class="stack-field">
                      <span class="field-label">{tr("create.documentLabel")}</span>
                      <select
                        class="source-input"
                        bind:value={draftEditorPath}
                        onchange={(event) => handleDraftDocumentSelect((event.currentTarget as HTMLSelectElement).value)}
                      >
                        {#each createPreview.documents as document}
                          <option value={document.path}>{document.path}</option>
                        {/each}
                      </select>
                    </label>
                    <div class="stack-field">
                      <span class="field-label">{tr("create.documentActions")}</span>
                      <div class="button-row">
                        <button disabled={busy} onclick={handleSaveDraftDocument}>
                          {tr("create.saveDocument")}
                        </button>
                      </div>
                    </div>
                  </div>
                  <textarea
                    class="editor-textarea"
                    bind:value={draftEditorContent}
                    rows="14"
                  ></textarea>
                {/if}
                <div class="list">
                  {#each createPreview.files as entry}
                    <div class="check-row info">
                      <strong>{entry.entry_kind}</strong>
                      <span>{createPreview.draft.version_id}</span>
                      <p>{entry.path}</p>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </section>
        </div>
      {/if}

      {#if activeTab === "benchmarks"}
        <div in:fade={{ duration: 200, delay: 100 }}>
          <section class="section">
            <div class="section-heading">
              <div>
                <h2>{tr("benchmarks.suitesTitle")}</h2>
                <p>{tr("benchmarks.suitesCopy")}</p>
              </div>
            </div>
            <div class="panel stack-panel" style="margin-bottom: 20px;">
              <div class="stat">
                <span>{tr("benchmarks.currentSource")}</span>
                <strong>{sourceInput}</strong>
              </div>
              <label class="stack-field">
                <span class="field-label">{tr("benchmarks.modeLabel")}</span>
                <select class="source-input" bind:value={benchmarkMode}>
                  <option value="deterministic">{tr("benchmarks.modeDeterministic")}</option>
                  <option value="human-review">{tr("benchmarks.modeHumanReview")}</option>
                  <option value="ai-judge">{tr("benchmarks.modeAiJudge")}</option>
                </select>
              </label>
              {#if benchmarkMode === "ai-judge"}
                <label class="stack-field">
                  <span class="field-label">{tr("benchmarks.executorModelLabel")}</span>
                  <input class="source-input" bind:value={aiExecutorModel} placeholder={tr("benchmarks.executorModelPlaceholder")} />
                </label>
              {/if}
              <div class="button-row">
                {#each benchmarkSuites as suite}
                  <button class="primary" disabled={busy} onclick={() => handleBenchmarkRun(suite.id)}>
                    <BookOpen size={16} /> {tr("benchmarks.runSuite", { title: suite.title })}
                  </button>
                {/each}
              </div>
            </div>
            <div class="list">
              {#each benchmarkSuites as suite}
                <div class="check-row info">
                  <strong>{suite.title}</strong>
                  <span>{suite.suite_kind}</span>
                  <p>{suite.description}</p>
                  <p>{tr("benchmarks.caseCount", { count: suite.case_count })}</p>
                </div>
              {/each}
            </div>
          </section>

          <section class="section">
            <div class="section-heading">
              <div>
                <h2>{tr("benchmarks.jobsTitle")}</h2>
                <p>{tr("benchmarks.jobsCopy")}</p>
              </div>
              <button disabled={busy} onclick={handleWorkJobs}>
                <RefreshCw size={14} /> {tr("benchmarks.workJobs")}
              </button>
            </div>

            {#if recentJobs.length}
              <div class="list">
                {#each recentJobs as job}
                  <div class="check-row info">
                    <strong>{job.kind}</strong>
                    <span>{job.status}</span>
                    <p>{job.summary}</p>
                    <p>{job.subject_id}</p>
                    <div class="button-row compact">
                      <button disabled={busy} onclick={() => handleRetryJob(job.id)}>{tr("benchmarks.retryJob")}</button>
                      <button disabled={busy} onclick={() => handleCancelJob(job.id)}>{tr("benchmarks.cancelJob")}</button>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="empty-inline">
                <p>{tr("benchmarks.noJobsYet")}</p>
              </div>
            {/if}
          </section>

          <section class="section">
            <div class="section-heading">
              <div>
                <h2>{tr("benchmarks.recentRunsTitle")}</h2>
                <p>{tr("benchmarks.recentRunsCopy")}</p>
              </div>
            </div>

            {#if recentBenchmarkRuns.length}
              <div class="list">
                {#each recentBenchmarkRuns as run}
                  <div class="check-row info">
                    <strong>{run.suite_id}</strong>
                    <span>{run.recommendation}</span>
                    <p>{run.summary}</p>
                    <p>{tr("benchmarks.score", { score: run.score.toFixed(1) })}</p>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="empty-inline">
                <p>{tr("benchmarks.noRunsYet")}</p>
              </div>
            {/if}
          </section>

          {#if pendingHumanRuns.length}
            <section class="section">
              <div class="section-heading">
                <div>
                  <h2>{tr("benchmarks.reviewQueueTitle")}</h2>
                  <p>{tr("benchmarks.reviewQueueCopy")}</p>
                </div>
              </div>
              <div class="panel stack-panel">
                <label class="stack-field">
                  <span class="field-label">{tr("benchmarks.reviewDecisionLabel")}</span>
                  <select class="source-input" bind:value={reviewDecision}>
                    <option value="promote">{tr("benchmarks.decisionPromote")}</option>
                    <option value="hold">{tr("benchmarks.decisionHold")}</option>
                    <option value="reject">{tr("benchmarks.decisionReject")}</option>
                    <option value="manual_review">{tr("benchmarks.decisionManualReview")}</option>
                  </select>
                </label>
                <label class="stack-field">
                  <span class="field-label">{tr("benchmarks.reviewNoteLabel")}</span>
                  <textarea class="source-input" rows="4" bind:value={reviewNote}></textarea>
                </label>
                <div class="list">
                  {#each pendingHumanRuns as run}
                    <div class="check-row info">
                      <strong>{run.status}</strong>
                      <span>{run.suite_id}</span>
                      <p>{run.summary}</p>
                      <div class="button-row compact">
                        <button class="primary" disabled={busy} onclick={() => handleSubmitHumanReview(run.id)}>
                          {tr("benchmarks.submitReview")}
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            </section>
          {/if}

          {#if benchmarkResult}
            <section class="section">
              <div class="section-heading">
                <div>
                  <h2>{tr("benchmarks.latestRunTitle")}</h2>
                  <p>{tr("benchmarks.latestRunCopy")}</p>
                </div>
              </div>
              <div class="list">
                <div class="check-row info">
                  <strong>{benchmarkResult.status}</strong>
                  <span>{benchmarkResult.recommendation}</span>
                  <p>{benchmarkResult.summary}</p>
                  <p>{tr("benchmarks.score", { score: benchmarkResult.score.toFixed(1) })}</p>
                </div>
              </div>
            </section>
          {/if}
        </div>
      {/if}

      {#if activeTab === "connections"}
        <div in:fade={{ duration: 200, delay: 100 }}>
          <section class="section empty">
            <ArrowRight size={48} color="rgba(255,255,255,0.2)" style="margin-bottom: 16px;" />
            <h2>{tr("connections.emptyTitle")}</h2>
            <p>{tr("connections.emptyCopy")}</p>
          </section>
        </div>
      {/if}

      {#if activeTab === "health"}
        <div in:fade={{ duration: 200, delay: 100 }}>
          <section class="section">
            <div class="alert note" style="align-items: center; justify-content: space-between; margin-bottom: 32px;">
              <div>{tr("doctor.intro")}</div>
              <button class="primary" disabled={busy} onclick={handleDoctor}>
                <Activity size={16} /> {tr("doctor.run")}
              </button>
            </div>

            {#if report}
              <div class="list">
                {#each report.checks as check}
                  <div class="check-row {check.level.toLowerCase()}">
                    <strong>{check.level}</strong>
                    <span>{check.code}</span>
                    <p>{check.message}</p>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="section empty">
                <Activity size={48} color="rgba(255,255,255,0.2)" style="margin-bottom: 16px;" />
                <h2>{tr("doctor.emptyTitle")}</h2>
                <p>{tr("doctor.emptyCopy")}</p>
              </div>
            {/if}
          </section>
        </div>
      {/if}

      {#if activeTab === "settings"}
        <div in:fade={{ duration: 200, delay: 100 }}>
          <section class="section">
              <div class="section-head">
                <div>
                  <h2>{tr("settings.languageTitle")}</h2>
                  <div class="meta">{tr("settings.languageCopy")}</div>
                </div>
              <div class="meta">{tr("settings.languageCurrent")}: {localeLabel(locale)}</div>
            </div>

            <div class="guide-slot-list">
              {#each LANGUAGE_OPTIONS as option}
                <button class:selected={locale === option.value} class="guide-slot" onclick={() => (locale = option.value)}>
                  <div>
                    <div class="eyebrow">{tr("settings.languageTitle")}</div>
                    <strong>{localeLabel(option.value)}</strong>
                    <p>{tr("settings.languagePersistence")}</p>
                  </div>
                  <div class="guide-slot-meta">
                    <span>{locale === option.value ? tr("common.selected") : tr("common.open")}</span>
                  </div>
                </button>
              {/each}
            </div>
          </section>

          <section class="section">
            <div class="receipt-panel">
              <div class="receipt-row"><span>{tr("settings.languageCurrent")}</span><strong>{localeLabel(locale)}</strong></div>
              <div class="receipt-row"><span>{tr("settings.supportedLanguages")}</span><strong>{LANGUAGE_OPTIONS.map((option) => localeLabel(option.value)).join(", ")}</strong></div>
              <div class="receipt-row"><span>{tr("settings.previewTitle")}</span><strong>{tr("settings.previewCopy")}</strong></div>
              <div class="receipt-row"><span>{tr("common.save")}</span><strong>{tr("settings.languagePersistence")}</strong></div>
            </div>
          </section>
        </div>
      {/if}
    </div>
  </main>
</div>

<style>
  :global(.spin) {
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    100% {
      transform: rotate(360deg);
    }
  }

  .error-btn:hover {
    background: rgba(255, 69, 58, 0.1) !important;
    border-color: rgba(255, 69, 58, 0.6) !important;
    color: #ff6961 !important;
  }
</style>
