<script lang="ts">
  import type { DeckInfo } from "../types";

  interface Props {
    deck: DeckInfo;
    selected?: boolean;
    recommended?: boolean;
    recommendedLabel: string;
    deckLabel: string;
    synthesizedLabel: string;
    declaredLabel: string;
    cardsInsideLabel: string;
    moreLabel: (count: number) => string;
    onclick?: () => void;
  }

  let {
    deck,
    selected = false,
    recommended = false,
    recommendedLabel,
    deckLabel,
    synthesizedLabel,
    declaredLabel,
    cardsInsideLabel,
    moreLabel,
    onclick = undefined,
  }: Props = $props();

  const previewSkills = $derived.by(() => deck.skills.slice(0, 3));
</script>

<button class:selected class:recommended class="deck-object" onclick={onclick}>
  <div class="stack stack-back"></div>
  <div class="stack stack-mid"></div>
  <div class="deck-surface">
    <div class="deck-top">
      <span class="deck-label">{recommended ? recommendedLabel : deckLabel}</span>
      <span class="deck-badge">{deck.synthesized ? synthesizedLabel : declaredLabel}</span>
    </div>
    <h3>{deck.name}</h3>
    <p>{deck.description}</p>
    <div class="deck-meta">
      <strong>{deck.skills.length}</strong>
      <span>{cardsInsideLabel}</span>
    </div>
    <div class="preview-list">
      {#each previewSkills as skill}
        <span>{skill}</span>
      {/each}
      {#if deck.skills.length > previewSkills.length}
        <span>{moreLabel(deck.skills.length - previewSkills.length)}</span>
      {/if}
    </div>
  </div>
</button>

<style>
  .deck-object {
    position: relative;
    border: none;
    background: transparent;
    padding: 0;
    text-align: left;
    min-height: 0;
  }

  .stack {
    position: absolute;
    inset: 0;
    border-radius: 24px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    transition: transform 0.25s ease, background 0.25s ease, border-color 0.25s ease;
  }

  .stack-back {
    background: rgba(255, 255, 255, 0.04);
    transform: translate(12px, 12px) rotate(4deg);
  }

  .stack-mid {
    background: rgba(255, 255, 255, 0.06);
    transform: translate(6px, 6px) rotate(2deg);
  }

  .deck-surface {
    position: relative;
    z-index: 2;
    min-height: 280px;
    padding: 28px;
    border-radius: 24px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02)),
      rgba(16, 18, 22, 0.92);
    box-shadow: 0 18px 40px rgba(0, 0, 0, 0.24);
    transition: transform 0.25s ease, border-color 0.25s ease, background 0.25s ease;
  }

  .deck-object:hover .deck-surface {
    transform: translateY(-4px);
    border-color: rgba(41, 151, 255, 0.45);
  }

  .deck-object:hover .stack-mid {
    transform: translate(9px, 8px) rotate(3deg);
    background: rgba(41, 151, 255, 0.12);
  }

  .deck-object:hover .stack-back {
    transform: translate(16px, 14px) rotate(5deg);
    background: rgba(41, 151, 255, 0.08);
  }

  .deck-object.selected .deck-surface,
  .deck-object.selected .stack-mid,
  .deck-object.selected .stack-back {
    border-color: rgba(41, 151, 255, 0.5);
  }

  .deck-object.selected .deck-surface {
    background:
      linear-gradient(180deg, rgba(41, 151, 255, 0.18), rgba(255, 255, 255, 0.03)),
      rgba(14, 18, 24, 0.96);
  }

  .deck-object.recommended .deck-surface {
    background:
      linear-gradient(180deg, rgba(255, 214, 10, 0.12), rgba(255, 255, 255, 0.03)),
      rgba(16, 18, 22, 0.92);
  }

  .deck-top,
  .deck-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .deck-label,
  .deck-badge {
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.7);
  }

  .deck-badge {
    padding: 6px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
  }

  h3 {
    margin: 24px 0 10px;
    font-size: 28px;
    line-height: 1.05;
    letter-spacing: -0.03em;
  }

  p {
    margin: 0;
    color: rgba(255, 255, 255, 0.72);
    line-height: 1.55;
    min-height: 66px;
  }

  .deck-meta {
    margin-top: 24px;
    padding-top: 18px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }

  .deck-meta strong {
    font-size: 32px;
    letter-spacing: -0.04em;
  }

  .deck-meta span {
    color: rgba(255, 255, 255, 0.58);
  }

  .preview-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 16px;
  }

  .preview-list span {
    padding: 7px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.74);
    font-size: 12px;
  }
</style>
