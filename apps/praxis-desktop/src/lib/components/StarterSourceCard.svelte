<script lang="ts">
  import type { StarterSourcePreset } from "../starterSources";

  interface Props {
    source: StarterSourcePreset;
    description: string;
    audience: string;
    badge: string;
    featuredLabel?: string;
    actionLabel?: string;
    actionStateLabel?: string;
    selected?: boolean;
    compact?: boolean;
    staticCard?: boolean;
    onclick?: () => void;
  }

  let {
    source,
    description,
    audience,
    badge,
    featuredLabel = "Featured",
    actionLabel = "Inspect source",
    actionStateLabel = "Open",
    selected = false,
    compact = false,
    staticCard = false,
    onclick = undefined,
  }: Props = $props();
</script>

<button class:selected class:compact class:featured={source.featured} class:static-card={staticCard} class="starter-card" onclick={onclick}>
  <div class="starter-card-top">
    <span class="starter-badge">{badge}</span>
    {#if source.featured}
      <span class="starter-badge accent">{featuredLabel}</span>
    {/if}
  </div>
  <h3>{source.title}</h3>
  <p>{description}</p>
  <div class="starter-audience">{audience}</div>
  <div class="starter-url">{source.url}</div>
  <div class="starter-action">
    <span>{actionLabel}</span>
    <strong>{actionStateLabel}</strong>
  </div>
</button>

<style>
  .starter-card {
    width: 100%;
    padding: 18px;
    border-radius: 18px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.07), rgba(255, 255, 255, 0.02)),
      rgba(18, 20, 24, 0.7);
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
    position: relative;
    overflow: hidden;
  }

  .starter-card.featured {
    background:
      linear-gradient(160deg, rgba(255, 214, 10, 0.14), rgba(41, 151, 255, 0.06) 55%, rgba(255, 255, 255, 0.02)),
      rgba(18, 20, 24, 0.82);
    border-color: rgba(255, 214, 10, 0.28);
  }

  .starter-card.selected {
    border-color: rgba(41, 151, 255, 0.48);
    box-shadow: 0 0 0 1px rgba(41, 151, 255, 0.18) inset;
  }

  .starter-card:not(.static-card):hover {
    transform: translateY(-2px);
    border-color: rgba(41, 151, 255, 0.38);
    box-shadow: 0 16px 32px rgba(0, 0, 0, 0.22);
  }

  .starter-card.static-card {
    cursor: default;
  }

  .starter-card.compact {
    padding: 15px;
    gap: 10px;
  }

  .starter-card-top {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .starter-badge {
    display: inline-flex;
    align-items: center;
    padding: 5px 9px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.78);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .starter-badge.accent {
    background: rgba(41, 151, 255, 0.16);
    color: rgba(255, 255, 255, 0.9);
  }

  .starter-card::after {
    content: "";
    position: absolute;
    inset: auto -18% -36% auto;
    width: 160px;
    height: 160px;
    border-radius: 999px;
    background: radial-gradient(circle, rgba(41, 151, 255, 0.18), transparent 62%);
    pointer-events: none;
  }

  h3 {
    margin: 0;
    font-size: 18px;
    letter-spacing: -0.02em;
  }

  p {
    margin: 0;
    color: rgba(255, 255, 255, 0.76);
    line-height: 1.55;
    min-height: 44px;
  }

  .compact p {
    min-height: 0;
    font-size: 13px;
  }

  .starter-audience {
    color: rgba(255, 255, 255, 0.62);
    font-size: 12px;
  }

  .starter-url {
    color: rgba(255, 255, 255, 0.42);
    font-size: 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    word-break: break-all;
  }

  .starter-action {
    margin-top: auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-top: 14px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    font-size: 12px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.56);
  }

  .starter-action strong {
    color: rgba(255, 255, 255, 0.88);
    font-size: 11px;
  }
</style>
