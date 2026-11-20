<script lang="ts">
  import { flow } from '$lib/flow.svelte';
  import TopBar from '$lib/components/TopBar.svelte';
</script>

<TopBar />
<main class="home">
  <h1>Welcome to Healing Hideaway</h1>
  <p class="muted">Take a breath. Booking takes about a minute.</p>

  <button class="tile primary" onclick={() => (flow.screen = 'service')}>
    <span class="display tile-title">Book a massage</span>
    <span>Pick a treatment, a time and you're set</span>
  </button>
  <!-- Gift card tiles come back once that's decided (see the progress file).
       Staff sign in with the Staff button in the top bar. -->
</main>

<style>
  .home {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.2rem;
    text-align: center;
  }
  h1 { font-size: 3rem; }
  .tile {
    margin-top: 1.5rem;
    width: min(100%, 30rem);
    min-height: 10rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    border-radius: 1.4rem;
  }
  .tile-title { font-size: 2rem; }
  .tile span:last-child { font-weight: 400; }
</style>
