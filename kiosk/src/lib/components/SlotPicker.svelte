<!-- Start times for flow.date. Unavailable times show no reason and no names. -->
<script lang="ts">
  import { api, type Slot } from '$lib/api';
  import { flow } from '$lib/flow.svelte';

  let slots = $state<Slot[] | null>(null); // null = loading
  let error = $state('');
  let request = 0;

  $effect(() => {
    const date = flow.date;
    const minutes = flow.minutes;
    if (!date) return;
    const mine = ++request;
    slots = null;
    error = '';
    api
      .daySlots(date, minutes)
      .then((s) => {
        if (mine === request) slots = s; // ignore answers for a day no longer selected
      })
      .catch((e) => {
        if (mine === request) error = String(e);
      });
  });

  const openCount = $derived(slots?.filter((s) => s.status === 'open').length ?? 0);

  function pick(s: Slot) {
    if (s.status !== 'open') return;
    flow.start = s.start;
    flow.startLabel = s.label;
  }
</script>

{#if !flow.date}
  <p class="muted">Pick a day to see times.</p>
{:else if error}
  <p role="alert">Could not load times: {error}</p>
{:else if slots === null}
  <p class="muted" aria-live="polite">Loading times…</p>
{:else if slots.length === 0}
  <p class="muted">Closed this day. Please pick another date.</p>
{:else}
  {#if openCount === 0}
    <p class="muted" style="margin-bottom:0.8rem">No times left this day. Please pick another date.</p>
  {/if}
  <div class="slot-grid">
    {#each slots as s (s.start)}
      {#if s.status === 'unavailable'}
        <button class="slot unavailable" disabled aria-label="{s.label}, unavailable">
          <span class="time">{s.label}</span><br /><small>Unavailable</small>
        </button>
      {:else}
        {@const on = flow.start === s.start}
        <button class="slot choice" class:on aria-pressed={on} onclick={() => pick(s)}>
          {#if on}<span aria-hidden="true">✓ </span>{/if}{s.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  /* Colours come from the shared `.choice` style in app.css */
  .slot { min-height: 4.2rem; padding: 0.4rem; line-height: 1.2; }
  .slot.choice { font-size: 1.15rem; }
</style>
