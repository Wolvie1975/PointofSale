<script lang="ts">
  import { flow, type TextSize } from '$lib/flow.svelte';

  const sizes: { value: TextSize; label: string; sample: string }[] = [
    { value: 'small', label: 'Small', sample: '0.9em' },
    { value: 'medium', label: 'Medium', sample: '1.1em' },
    { value: 'large', label: 'Large', sample: '1.35em' },
  ];

  const close = () => (flow.textSizeOpen = false);

  /** Move focus to the current choice when the picker opens. */
  function focusSelected(node: HTMLElement) {
    node.querySelector<HTMLElement>('[aria-checked="true"]')?.focus();
  }
  // Esc is handled by the screen router (it closes this picker first).
</script>

<!-- Tapping anywhere outside the panel closes it -->
<button class="backdrop" aria-label="Close text size" tabindex="-1" onclick={close}></button>

<div class="panel" role="dialog" aria-label="Text size" use:focusSelected>
  <p class="title">Text size</p>
  <div role="radiogroup" aria-label="Text size" class="options">
    {#each sizes as s (s.value)}
      <button
        role="radio"
        aria-checked={flow.textSize === s.value}
        class:primary={flow.textSize === s.value}
        onclick={() => { flow.textSize = s.value; close(); }}
      >
        <span class="display" style:font-size={s.sample}>Aa</span>
        {s.label}
      </button>
    {/each}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.35);
    border: none;
    border-radius: 0;
    min-height: 0;
    padding: 0;
    cursor: default;
    z-index: 10;
  }
  .backdrop:hover { box-shadow: none; filter: none; }
  .panel {
    position: fixed;
    top: 4.8rem;
    right: 2.5rem;
    z-index: 11;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    border-radius: 1rem;
    padding: 1rem;
    width: min(28rem, calc(100vw - 2rem));
  }
  .title { font-weight: 700; margin-bottom: 0.75rem; }
  .options { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.5rem; }
  .options button { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.2rem; padding: 0.6rem; }
</style>
