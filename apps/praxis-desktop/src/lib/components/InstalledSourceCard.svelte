<script lang="ts">
  import type { AppliedInstall, SourceInstall } from "../types";

  interface Props {
    install: SourceInstall;
    applied?: AppliedInstall | null;
    current?: boolean;
    labels: {
      currentSource: string;
      emptySelection: string;
      appliedSkills: string;
      agentFileActions: string;
      bundles: string;
      reference: string;
      local: string;
      selectionAll: string;
      selectionDecks: (count: number) => string;
      selectionCards: (count: number) => string;
      selectionAgentFileTemplates: (count: number) => string;
      metaExcluded: (items: string) => string;
      metaAgentFileTemplates: (items: string) => string;
      metaSourceHash: (hash: string) => string;
    };
  }

  let { install, applied = null, current = false, labels }: Props = $props();

  const summary = $derived.by(() => {
    const selection = install.selection;
    const parts: string[] = [];
    if (selection.all) parts.push(labels.selectionAll);
    if (selection.decks.length) parts.push(labels.selectionDecks(selection.decks.length));
    if (selection.skills.length) parts.push(labels.selectionCards(selection.skills.length));
    if (selection.agent_file_templates.length) {
      parts.push(labels.selectionAgentFileTemplates(selection.agent_file_templates.length));
    }
    return parts.join(" · ");
  });

  const sourceLabel = $derived.by(() => {
    if (install.source.kind === "github") {
      return `${install.source.owner}/${install.source.repo}`;
    }
    return install.source.path;
  });
</script>

<div class:current class="source-shelf">
  <div class="shelf-main">
    <div class="shelf-head">
      <div>
        <div class="shelf-label">{sourceLabel}</div>
        <strong>{install.id}</strong>
      </div>
      <div class="pill-row">
        {#if current}
          <span class="pill active">{labels.currentSource}</span>
        {/if}
        {#each install.targets as target}
          <span class="pill">{target}</span>
        {/each}
      </div>
    </div>

    <div class="shelf-summary">{summary || labels.emptySelection}</div>

    <div class="shelf-grid">
      <div>
        <span>{labels.appliedSkills}</span>
        <strong>{applied?.skills.length ?? 0}</strong>
      </div>
      <div>
        <span>{labels.agentFileActions}</span>
        <strong>{applied?.agent_file_actions.length ?? 0}</strong>
      </div>
      <div>
        <span>{labels.bundles}</span>
        <strong>{applied?.bundles.length ?? 0}</strong>
      </div>
      <div>
        <span>{labels.reference}</span>
        <strong>{applied?.resolved_reference ?? labels.local}</strong>
      </div>
    </div>

    {#if install.selection.exclude_skills.length}
      <div class="shelf-meta">{labels.metaExcluded(install.selection.exclude_skills.join(", "))}</div>
    {/if}

    {#if install.selection.agent_file_templates.length}
      <div class="shelf-meta">{labels.metaAgentFileTemplates(install.selection.agent_file_templates.join(", "))}</div>
    {/if}

    {#if applied?.source_hash}
      <div class="shelf-meta">{labels.metaSourceHash(applied.source_hash.slice(0, 12))}</div>
    {/if}
  </div>
</div>

<style>
  .source-shelf {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 24px;
    padding: 22px 24px;
    background:
      radial-gradient(circle at top right, rgba(41, 151, 255, 0.12), transparent 24%),
      linear-gradient(180deg, rgba(255,255,255,0.06), rgba(255,255,255,0.02)),
      rgba(18, 20, 24, 0.8);
    backdrop-filter: blur(14px);
    position: relative;
    overflow: hidden;
  }

  .source-shelf.current {
    border-color: rgba(41, 151, 255, 0.42);
    box-shadow: inset 0 0 0 1px rgba(41, 151, 255, 0.12);
  }

  .shelf-main {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .shelf-head {
    display: flex;
    justify-content: space-between;
    gap: 18px;
    align-items: flex-start;
  }

  .shelf-label {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.58);
    margin-bottom: 6px;
    word-break: break-all;
  }

  strong {
    font-size: 18px;
    letter-spacing: -0.02em;
  }

  .pill-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .pill {
    padding: 6px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.72);
    font-size: 12px;
  }

  .pill.active {
    background: rgba(41, 151, 255, 0.18);
    color: #9ed0ff;
  }

  .shelf-summary,
  .shelf-meta {
    color: rgba(255, 255, 255, 0.72);
    line-height: 1.5;
  }

  .shelf-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 12px;
  }

  .shelf-grid div {
    padding: 14px 16px;
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(255, 255, 255, 0.04);
  }

  .shelf-grid span {
    display: block;
    margin-bottom: 8px;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.56);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .shelf-grid strong {
    display: block;
    font-size: 14px;
    word-break: break-all;
  }
</style>
