<script lang="ts">
  import { flow, resetFlow } from '$lib/flow.svelte';
  import TextSizePicker from './TextSizePicker.svelte';

  /** `step` is omitted on Home and Done. */
  let { step, label }: { step?: number; label?: string } = $props();

  // Hidden staff entry: press and hold the name for 3 s to open the staff PIN pad.
  let holdTimer: ReturnType<typeof setTimeout> | undefined;
  function startHold() {
    clearTimeout(holdTimer);
    holdTimer = setTimeout(() => {
      flow.textSizeOpen = false;
      flow.staffOpen = true;
    }, 3000);
  }
  const cancelHold = () => clearTimeout(holdTimer);
</script>

<header>
  <span
    class="brand display"
    role="presentation"
    onpointerdown={startHold}
    onpointerup={cancelHold}
    onpointerleave={cancelHold}
    onpointercancel={cancelHold}>Healing Hideaway</span
  >

  {#if step}
    <span class="step" aria-live="polite">Step {step} of 3 · {label}</span>
  {:else}
    <span></span>
  {/if}

  <span class="actions">
    {#if flow.screen !== 'home'}
      <button class="ghost" onclick={resetFlow}>Start over</button>
    {/if}
    <button
      class="ghost"
      aria-haspopup="dialog"
      aria-expanded={flow.textSizeOpen}
      onclick={() => (flow.textSizeOpen = !flow.textSizeOpen)}
    >
      <span aria-hidden="true" class="aa">Aa</span> Text size
    </button>
  </span>
</header>

{#if flow.textSizeOpen}
  <TextSizePicker />
{/if}

<style>
  header {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 2.5rem;
    border-bottom: 1px solid var(--border);
  }
  .brand { font-size: 1.5rem; }
  .step { color: var(--muted); }
  .actions { display: flex; gap: 0.5rem; justify-content: flex-end; }
  .aa { font-family: var(--font-display); font-size: 1.2em; margin-right: 0.3rem; }
</style>
