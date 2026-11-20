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
  import StaffClose from '$lib/components/StaffClose.svelte';

  onMount(() => startIdleTimer(90_000));

  // Esc = "go back one level": close the staff PIN pad or text size picker,
  // then hide the on-screen keyboard, then step back a screen (Done → Home).
  // Ctrl+Alt+S opens the staff PIN pad (same as holding the name in the top bar).
  function onKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.altKey && e.key.toLowerCase() === 's') {
      e.preventDefault();
      flow.textSizeOpen = false;
      flow.staffOpen = true;
      return;
    }
    if (e.key !== 'Escape') return;
    e.preventDefault();
    if (flow.staffOpen) {
      flow.staffOpen = false;
    } else if (flow.textSizeOpen) {
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
  {#if flow.staffOpen}<StaffClose />{/if}
</div>
