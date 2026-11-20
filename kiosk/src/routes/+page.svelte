<!-- Screen router: one screen at a time, chosen by flow.screen. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { backFrom, flow, resetFlow } from '$lib/flow.svelte';
  import { startIdleTimer } from '$lib/idle';
  import Home from '$lib/screens/Home.svelte';
  import ServiceStep from '$lib/screens/ServiceStep.svelte';
  import TimeStep from '$lib/screens/TimeStep.svelte';
  import CalendarStep from '$lib/screens/CalendarStep.svelte';
  import DetailsStep from '$lib/screens/DetailsStep.svelte';
  import DoneStep from '$lib/screens/DoneStep.svelte';

  onMount(() => startIdleTimer(90_000));

  // Esc = "go back one level": close the text size picker, then hide the
  // on-screen keyboard, then step back a screen (Done → Home).
  function onKeydown(e: KeyboardEvent) {
    if (e.key !== 'Escape') return;
    e.preventDefault();
    if (flow.textSizeOpen) {
      flow.textSizeOpen = false;
    } else if (flow.activeField) {
      flow.activeField = null;
      (document.activeElement as HTMLElement | null)?.blur();
    } else if (flow.screen === 'done') {
      resetFlow();
    } else {
      const to = backFrom[flow.screen];
      if (to) flow.screen = to;
    }
  }

  // Each new screen starts scrolled to the top with focus at the start,
  // so keyboard users don't land mid-page.
  $effect(() => {
    void flow.screen;
    (document.activeElement as HTMLElement | null)?.blur();
    document.querySelector('.kiosk main')?.scrollTo(0, 0);
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="kiosk">
  {#if flow.screen === 'home'}<Home />
  {:else if flow.screen === 'service'}<ServiceStep />
  {:else if flow.screen === 'time'}<TimeStep />
  {:else if flow.screen === 'calendar'}<CalendarStep />
  {:else if flow.screen === 'details'}<DetailsStep />
  {:else if flow.screen === 'done'}<DoneStep />
  {/if}
</div>
