<script lang="ts">
  interface Props {
    title: string;
    eyebrow: string;
    description: string;
    badge?: string | null;
    selected?: boolean;
    secondary?: string | null;
    footerTag?: string | null;
    staticCard?: boolean;
    onclick?: () => void;
  }

  let {
    title,
    eyebrow,
    description,
    badge = null,
    selected = false,
    secondary = null,
    footerTag = "skill",
    staticCard = false,
    onclick = undefined,
  }: Props = $props();
</script>

<button class:selected class:static-card={staticCard} class="card skill-card" onclick={onclick}>
  <div class="card-glow"></div>
  <div class="card-top">
    <span class="eyebrow">{eyebrow}</span>
    {#if badge}
      <span class="badge subtle">{badge}</span>
    {/if}
  </div>
  <h3>{title}</h3>
  <p>{description}</p>
  {#if secondary || footerTag}
    <div class="card-footer">
      <div class="secondary">{secondary}</div>
      {#if footerTag}
        <span class="card-rank">{footerTag}</span>
      {/if}
    </div>
  {/if}
</button>

<style>
  .skill-card {
    position: relative;
    overflow: hidden;
    min-height: 238px;
    border-radius: 22px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background:
      radial-gradient(circle at top left, rgba(41, 151, 255, 0.18), transparent 34%),
      linear-gradient(180deg, rgba(255, 255, 255, 0.07), rgba(255, 255, 255, 0.02)),
      rgba(16, 18, 22, 0.88);
    box-shadow: 0 18px 36px rgba(0, 0, 0, 0.18);
  }

  .skill-card.selected {
    border-color: rgba(41, 151, 255, 0.42);
    box-shadow:
      0 20px 40px rgba(0, 0, 0, 0.24),
      0 0 0 1px rgba(41, 151, 255, 0.16) inset;
  }

  .skill-card:hover {
    transform: translateY(-3px);
  }

  .skill-card.static-card {
    cursor: default;
  }

  .skill-card.static-card:hover {
    transform: none;
  }

  .card-glow {
    position: absolute;
    inset: auto -20% -45% auto;
    width: 180px;
    height: 180px;
    border-radius: 999px;
    background: radial-gradient(circle, rgba(255, 214, 10, 0.16), transparent 64%);
    pointer-events: none;
  }

  .card-footer {
    margin-top: auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-top: 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }

  .card-rank {
    display: inline-flex;
    align-items: center;
    padding: 6px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.78);
    font-size: 10px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }
</style>
