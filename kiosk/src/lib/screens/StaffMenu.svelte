<!-- Staff home, after signing in with the PIN. -->
<script lang="ts">
  import { isSignedOutError, staffApi } from '$lib/api';
  import { flow, resetFlow } from '$lib/flow.svelte';
  import TopBar from '$lib/components/TopBar.svelte';

  let confirmClose = $state(false);
  let error = $state('');

  async function closeApp() {
    if (!flow.staffToken) return resetFlow();
    try {
      await staffApi.closeApp(flow.staffToken); // the app exits on success
    } catch (e) {
      if (isSignedOutError(e)) resetFlow();
      else error = String(e);
    }
  }
</script>

<TopBar />
<main class="menu">
  <h1>Staff</h1>
  <p class="muted">Signed in. You'll be signed out after a few minutes without activity.</p>

  <div class="tiles">
    <button class="tile primary" onclick={() => (flow.screen = 'staffClients')}>
      <span class="display tile-title">Clients</span>
      <span>Private notes, bookings and Mark paid</span>
    </button>

    {#if confirmClose}
      <div class="tile confirm" role="alert">
        <span class="display tile-title">Close the app?</span>
        <span class="muted">On the office kiosk it restarts fresh, which clears all bookings and notes (POC).</span>
        <span class="row">
          <button onclick={() => (confirmClose = false)}>Cancel</button>
          <button class="primary" onclick={closeApp}>Close app</button>
        </span>
      </div>
    {:else}
      <button class="tile" onclick={() => (confirmClose = true)}>
        <span class="display tile-title">Close app</span>
        <span class="muted">Quit the kiosk app</span>
      </button>
    {/if}
  </div>

  {#if error}<p role="alert">{error}</p>{/if}

  <button class="ghost" onclick={resetFlow}>Sign out</button>
</main>

<style>
  .menu { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 1.2rem; text-align: center; }
  h1 { font-size: 2.8rem; }
  .tiles { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1.2rem; width: min(100%, 52rem); margin-top: 1rem; }
  :global(html[data-text-size='large']) .tiles { grid-template-columns: minmax(0, 1fr); }
  .tile {
    min-height: 10rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 1rem 1.4rem;
    border-radius: 1.4rem;
  }
  .tile-title { font-size: 2rem; }
  .tile.primary span:last-child { font-weight: 400; }
  .confirm { background: var(--surface); border: 2px solid var(--accent); }
  .row { display: flex; gap: 0.6rem; margin-top: 0.4rem; }
</style>
