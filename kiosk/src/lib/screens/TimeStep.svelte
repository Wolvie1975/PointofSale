<script lang="ts">
  import { flow } from '$lib/flow.svelte';
  import StepPlaceholder from '$lib/components/StepPlaceholder.svelte';
</script>

<StepPlaceholder step={2} label="Time" title="When works for you?" next="details" nextLabel="Continue to your details">
  {#snippet extra()}
    <button onclick={() => (flow.screen = 'calendar')}>See full calendar ›</button>
  {/snippet}
</StepPlaceholder>
