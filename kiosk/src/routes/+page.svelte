<!-- TEMPORARY (Phase 2 check): proves the UI can call the Rust backend.
     Phase 3 replaces this with the real screen router. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { api, formatPrice, type Service } from '$lib/api';

  let services = $state<Service[]>([]);
  let error = $state('');

  onMount(async () => {
    try {
      services = await api.services();
    } catch (e) {
      error = String(e);
    }
  });
</script>

<main>
  <h1>Healing Hideaway</h1>
  <p class="muted">Phase 2 check: services loaded from the Rust backend</p>

  {#if error}
    <p role="alert">Could not load services: {error}</p>
  {:else if services.length === 0}
    <p class="muted">Loading…</p>
  {:else}
    <ul>
      {#each services as s (s.id)}
        <li>
          <strong>{s.name}</strong> — {s.description}
          <div class="muted">
            {s.options.map((o) => `${o.minutes} min · ${formatPrice(o.priceCents)}`).join('  |  ')}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</main>

<style>
  :global(body) { margin: 0; background: #16151b; color: #f3eee6; font-family: system-ui, sans-serif; }
  main { padding: 2rem 2.5rem; }
  ul { list-style: none; padding: 0; display: grid; gap: 1rem; }
  li { background: #221f29; border: 1px solid #3a3542; border-radius: 0.9rem; padding: 1rem 1.25rem; }
  .muted { color: #b8b0a4; }
</style>
