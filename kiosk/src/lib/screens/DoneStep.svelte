<script lang="ts">
  import { onMount } from 'svelte';
  import { resetFlow } from '$lib/flow.svelte';
  import TopBar from '$lib/components/TopBar.svelte';

  // Back to Home for the next client after 30 s
  onMount(() => {
    const t = setTimeout(resetFlow, 30_000);
    return () => clearTimeout(t);
  });
</script>

<TopBar />
<main style="display:flex; flex-direction:column; align-items:center; justify-content:center; gap:1rem; text-align:center">
  <h1 style="font-size:2.8rem">You're booked</h1>
  <p class="muted">Placeholder: the booking summary is built in Phase 4.</p>
  <p class="muted">Payment is taken at your appointment.</p>
  <button class="primary" style="margin-top:1rem" onclick={resetFlow}>Done</button>
</main>
