<script lang="ts">
  import type { GuideState, GuideKind } from "../types";

  interface Props {
    guide?: GuideState | null;
    busy?: boolean;
    onSave?: (kind: GuideKind, content: string) => void;
    labels: {
      save: string;
      managedBlocks: (count: number) => string;
      exists: string;
      notCreated: string;
      emptyTitle: string;
      emptyCopy: string;
    };
  }

  let { guide = null, busy = false, onSave = undefined, labels }: Props = $props();
  let content = $state("");

  $effect(() => {
    content = guide?.user_content ?? "";
  });
</script>

{#if guide}
  <section class="editor">
    <div class="editor-head">
      <div>
        <h3>{guide.kind}</h3>
        <div class="meta">{guide.target_path}</div>
      </div>
      <button class="primary" disabled={busy} onclick={() => onSave?.(guide.kind, content)}>
        {labels.save}
      </button>
    </div>

    <textarea bind:value={content} rows="12" class="editor-textarea"></textarea>

    <div class="editor-meta">
      <span>{labels.managedBlocks(guide.managed_blocks.length)}</span>
      <span>{guide.exists ? labels.exists : labels.notCreated}</span>
    </div>

    {#if guide.managed_blocks.length}
      <pre class="code-block">{JSON.stringify(guide.managed_blocks, null, 2)}</pre>
    {/if}
  </section>
{:else}
  <section class="editor empty">
    <h3>{labels.emptyTitle}</h3>
    <p>{labels.emptyCopy}</p>
  </section>
{/if}
