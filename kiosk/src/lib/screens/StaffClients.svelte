<!-- Staff: client list with search. Clients are built from bookings. -->
<script lang="ts">
  import { isSignedOutError, staffApi, type ClientSummary } from '$lib/api';
  import { flow, resetFlow } from '$lib/flow.svelte';
  import { shortDateTime } from '$lib/dates';
  import TopBar from '$lib/components/TopBar.svelte';

  let query = $state('');
  let clients = $state<ClientSummary[] | null>(null);
  let error = $state('');
  let request = 0;

  // Search as you type (short pause so every key doesn't hit the backend)
  $effect(() => {
    const q = query;
    const token = flow.staffToken;
    if (!token) return;
    const mine = ++request;
    const t = setTimeout(async () => {
      try {
        const result = await staffApi.clients(token, q);
        if (mine === request) clients = result;
      } catch (e) {
        if (isSignedOutError(e)) resetFlow();
        else error = String(e);
      }
    }, 150);
    return () => clearTimeout(t);
  });

  function open(c: ClientSummary) {
    flow.staffClientId = c.id;
    flow.screen = 'staffClient';
  }

  const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? '' : 's'}`;
</script>

<TopBar label="Clients" />
<main>
  <div class="head">
    <h1 style="font-size:2.4rem">Clients</h1>
    <input type="search" aria-label="Search clients" placeholder="Search name, phone or email" bind:value={query} />
  </div>

  {#if error}
    <p role="alert">{error}</p>
  {:else if clients === null}
    <p class="muted">Loading…</p>
  {:else if clients.length === 0}
    <p class="muted">{query ? 'No clients match that search.' : 'No clients yet. They appear here after their first booking.'}</p>
  {:else}
    <ul class="list">
      {#each clients as c (c.id)}
        <li>
          <button class="row" onclick={() => open(c)}>
            <span class="who">
              <strong>{c.name || 'No name'}</strong>
              <span class="muted">{[c.phone, c.email].filter(Boolean).join(' · ')}</span>
            </span>
            <span class="facts">
              <span>{plural(c.bookings, 'booking')}</span>
              {#if c.unpaid > 0}<span class="badge unpaid">{c.unpaid} unpaid</span>{/if}
              <span>{plural(c.notes, 'note')}</span>
              {#if c.nextVisit}<span class="muted">Next: {shortDateTime(c.nextVisit)}</span>{/if}
            </span>
            <span aria-hidden="true" class="chev">›</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</main>
<footer>
  <button onclick={() => (flow.screen = 'staff')}>Back</button>
</footer>

<style>
  .head { display: flex; justify-content: space-between; align-items: center; gap: 1rem; flex-wrap: wrap; margin-bottom: 1.2rem; }
  .head input { width: min(100%, 26rem); }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.6rem; }
  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 1.2rem;
    text-align: left;
    padding: 0.8rem 1.2rem;
    flex-wrap: wrap;
  }
  .who { display: flex; flex-direction: column; gap: 0.2rem; flex: 1 1 14rem; min-width: 0; }
  .facts { display: flex; gap: 0.9rem; align-items: center; flex-wrap: wrap; }
  .chev { font-size: 1.6rem; color: var(--muted); }
  .badge { padding: 0.15rem 0.6rem; border-radius: 999px; font-size: 0.85rem; font-weight: 700; }
  .badge.unpaid { background: #4a3a1e; color: #f2d29b; }
</style>
