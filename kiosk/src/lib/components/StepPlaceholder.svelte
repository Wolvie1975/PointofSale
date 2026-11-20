<!-- TEMPORARY (Phase 3): stands in for a booking step until Phase 4 builds it. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { backFrom, flow, type Screen } from '$lib/flow.svelte';
  import TopBar from './TopBar.svelte';

  let {
    step,
    label,
    title,
    next,
    nextLabel,
    extra,
  }: { step: number; label: string; title: string; next: Screen; nextLabel: string; extra?: Snippet } =
    $props();

  const back = $derived(backFrom[flow.screen]);
</script>

<TopBar {step} {label} />
<main>
  <h1 style="font-size:2.4rem">{title}</h1>
  <p class="muted" style="margin-top:0.75rem">Placeholder: this step is built in Phase 4.</p>
  {#if extra}<div style="margin-top:1.5rem">{@render extra()}</div>{/if}
</main>
<footer>
  {#if back}<button onclick={() => (flow.screen = back)}>Back</button>{/if}
  <button class="primary" onclick={() => (flow.screen = next)}>{nextLabel}</button>
</footer>
