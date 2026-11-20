<script lang="ts">
  import { onMount } from 'svelte';
  import { formatPrice } from '$lib/api';
  import { flow, resetFlow } from '$lib/flow.svelte';
  import { longDate } from '$lib/dates';
  import TopBar from '$lib/components/TopBar.svelte';

  const c = flow.confirmation;

  // Back to Home for the next client after 30 s
  onMount(() => {
    const t = setTimeout(resetFlow, 30_000);
    return () => clearTimeout(t);
  });
</script>

<TopBar />
<main class="done">
  <h1>You're booked</h1>
  {#if c}
    <dl>
      <dt>Treatment</dt><dd>{c.serviceName}, {c.minutes} min</dd>
      <dt>When</dt><dd>{longDate(c.date)} at {c.startLabel}</dd>
      <dt>Price</dt><dd>{formatPrice(c.priceCents)}</dd>
    </dl>
  {/if}
  <p class="muted">Payment is taken at your appointment.</p>
  <button class="primary" onclick={resetFlow}>Done</button>
</main>

<style>
  .done { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 1.2rem; text-align: center; }
  h1 { font-size: 2.8rem; }
  dl {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: 0.6rem 1.5rem;
    margin: 0.5rem 0;
    padding: 1.2rem 1.6rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 1.2rem;
    text-align: left;
  }
  dt { color: var(--muted); }
  dd { margin: 0; font-weight: 700; }
  button { margin-top: 0.5rem; min-width: 12rem; }
</style>
