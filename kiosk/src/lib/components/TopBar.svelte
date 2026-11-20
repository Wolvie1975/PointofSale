<script lang="ts">
  import { flow, isStaffScreen, resetFlow } from '$lib/flow.svelte';
  import TextSizePicker from './TextSizePicker.svelte';

  /** `step` is for the booking steps; staff screens pass only `label`. */
  let { step, label }: { step?: number; label?: string } = $props();

  const staff = $derived(isStaffScreen(flow.screen));

  function openStaff() {
    flow.textSizeOpen = false;
    flow.staffSignInOpen = true;
  }
</script>

<header class:staff>
  <span class="brand display">Healing Hideaway</span>

  {#if step}
    <span class="step" aria-live="polite">Step {step} of 3 · {label}</span>
  {:else if staff}
    <span class="step staff-label" aria-live="polite">Staff{label ? ` · ${label}` : ''}</span>
  {:else}
    <span></span>
  {/if}

  <span class="actions">
    {#if staff}
      <button class="ghost" onclick={resetFlow}>Sign out</button>
    {:else}
      {#if flow.screen !== 'home'}
        <button class="ghost" onclick={resetFlow}>Start over</button>
      {/if}
      <!-- POC: staff entry is visible (still PIN-protected) -->
      <button class="ghost" aria-haspopup="dialog" onclick={openStaff}>Staff</button>
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
  /* Staff screens get a distinct band so it's obvious nobody is in the client flow */
  header.staff { background: #241d30; border-bottom-color: var(--accent); }
  .brand { font-size: 1.5rem; }
  .step { color: var(--muted); }
  .staff-label { color: var(--accent); font-weight: 700; }
  .actions { display: flex; gap: 0.5rem; justify-content: flex-end; }
  .aa { font-family: var(--font-display); font-size: 1.2em; margin-right: 0.3rem; }
</style>
