<script lang="ts">
  import { onMount } from 'svelte';
  import { api, formatPrice, type Minutes, type Service } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import StepFooter from '$lib/components/StepFooter.svelte';

  let services = $state<Service[]>([]);
  let error = $state('');

  onMount(async () => {
    try {
      services = await api.services();
    } catch (e) {
      error = String(e);
    }
  });

  const selected = $derived(services.find((s) => s.id === flow.serviceId));

  function choose(s: Service, minutes?: Minutes) {
    const m = minutes ?? (s.options.some((o) => o.minutes === flow.minutes) ? flow.minutes : s.options[0].minutes);
    if (s.id !== flow.serviceId || m !== flow.minutes) {
      flow.start = ''; // a different service or length changes which times fit
      flow.startLabel = '';
    }
    flow.serviceId = s.id;
    flow.minutes = m;
  }
</script>

<TopBar step={1} label="Service" />
<main>
  <h1 style="font-size:2.4rem">What would you like today?</h1>

  {#if error}
    <p role="alert" style="margin-top:1rem">Could not load services: {error}</p>
  {/if}

  <div class="cards">
    {#each services as s (s.id)}
      {@const isSelected = s.id === flow.serviceId}
      <div class="card" class:selected={isSelected}>
        <button class="pick" aria-pressed={isSelected} onclick={() => choose(s)}>
          <span class="display name">{s.name}</span>
          <span class="muted">{s.description}</span>
        </button>
        <div class="lengths" role="group" aria-label="{s.name} length">
          {#each s.options as o (o.minutes)}
            <button
              class="chip"
              class:primary={isSelected && flow.minutes === o.minutes}
              aria-pressed={isSelected && flow.minutes === o.minutes}
              onclick={() => choose(s, o.minutes)}
            >
              {o.minutes} min · {formatPrice(o.priceCents)}
            </button>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</main>
<StepFooter
  nextLabel="Continue to time"
  disabled={!selected}
  hint={selected ? `${selected.name}, ${flow.minutes} min` : 'Pick a treatment and length'}
  onNext={() => (flow.screen = 'time')}
/>

<style>
  .cards { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1rem; margin-top: 1.4rem; }
  :global(html[data-text-size='large']) .cards { grid-template-columns: minmax(0, 1fr); }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 1.2rem;
    padding: 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .card.selected { border: 2px solid var(--accent); }
  .pick {
    background: transparent;
    border: none;
    text-align: left;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.3rem;
    padding: 0.6rem 0.8rem;
  }
  .name { font-size: 1.6rem; }
  .lengths { display: flex; flex-wrap: wrap; gap: 0.5rem; padding: 0 0.6rem 0.4rem; }
  .chip { min-height: 3rem; padding: 0 1rem; background: var(--surface-2); }
</style>
