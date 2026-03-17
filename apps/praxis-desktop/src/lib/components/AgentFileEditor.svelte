<script lang="ts">
  import type { AgentFileSlot, AgentFileState } from "../types";

  interface Props {
    slotState?: AgentFileState | null;
    busy?: boolean;
    onSave?: (slot: AgentFileSlot, content: string) => void;
    labels: {
      save: string;
      managedBlocks: (count: number) => string;
      exists: string;
      notCreated: string;
      emptyTitle: string;
      emptyCopy: string;
    };
  }

  let { slotState = null, busy = false, onSave = undefined, labels }: Props = $props();
  let content = $state("");

  $effect(() => {
    content = slotState?.user_content ?? "";
  });
</script>

{#if slotState}
  <section class="editor">
    <div class="editor-head">
      <div>
        <h3>{slotState.slot}</h3>
        <div class="meta">{slotState.target_path}</div>
      </div>
      <button class="primary" disabled={busy} onclick={() => onSave?.(slotState.slot, content)}>
        {labels.save}
      </button>
    </div>

    <textarea bind:value={content} rows="12" class="editor-textarea"></textarea>

    <div class="editor-meta">
      <span>{labels.managedBlocks(slotState.managed_blocks.length)}</span>
      <span>{slotState.exists ? labels.exists : labels.notCreated}</span>
    </div>

    {#if slotState.managed_blocks.length}
      <pre class="code-block">{JSON.stringify(slotState.managed_blocks, null, 2)}</pre>
    {/if}
  </section>
{:else}
  <section class="editor empty">
    <h3>{labels.emptyTitle}</h3>
    <p>{labels.emptyCopy}</p>
  </section>
{/if}
