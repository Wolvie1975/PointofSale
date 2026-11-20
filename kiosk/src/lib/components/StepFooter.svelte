<!-- Back (same target as Esc) plus the step's main action. -->
<script lang="ts">
  import { backFrom, flow } from '$lib/flow.svelte';

  let {
    nextLabel,
    onNext,
    disabled = false,
    hint = '',
  }: { nextLabel: string; onNext: () => void; disabled?: boolean; hint?: string } = $props();

  const back = $derived(backFrom[flow.screen]);
</script>

<footer>
  {#if hint}<span class="hint muted">{hint}</span>{/if}
  {#if back}<button onclick={() => (flow.screen = back)}>Back</button>{/if}
  <button class="primary" {disabled} onclick={onNext}>{nextLabel}</button>
</footer>

<style>
  footer { align-items: center; }
  .hint { margin-right: auto; }
</style>
