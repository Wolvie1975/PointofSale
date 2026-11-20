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
  import StaffMenu from '$lib/screens/StaffMenu.svelte';
  import StaffClients from '$lib/screens/StaffClients.svelte';
  import StaffClient from '$lib/screens/StaffClient.svelte';
  import Keyboard from '$lib/components/Keyboard.svelte';
  import StaffSignIn from '$lib/components/StaffSignIn.svelte';

  onMount(() => startIdleTimer(90_000));

  // ---- On-screen keyboard: shown while a text field has focus ----
  const TEXT_TYPES = ['text', 'tel', 'email', 'search', ''];
  function keyboardFor(el: EventTarget | null) {
    if (el instanceof HTMLTextAreaElement && !el.readOnly) return 'text';
    if (el instanceof HTMLInputElement && !el.readOnly && TEXT_TYPES.includes(el.type)) {
      return el.type === 'tel' ? 'phone' : 'text';
    }
    return null;
  }
  // Inside the PIN pad, staff use its own digit keys
  const inDialog = (el: EventTarget | null) => el instanceof Element && !!el.closest('[role="dialog"]');

  function onFocusIn(e: FocusEvent) {
    if (!inDialog(e.target)) flow.keyboard = keyboardFor(e.target);
  }
  function onFocusOut(e: FocusEvent) {
    if (!keyboardFor(e.relatedTarget)) flow.keyboard = null;
  }
  // Tapping a field that already has focus brings the keyboard back after Hide
  function onPointerDown(e: PointerEvent) {
    if (e.target === document.activeElement && !inDialog(e.target)) flow.keyboard = keyboardFor(e.target);
  }

  // ---- Keys ----
  // Esc = "go back one level": close the PIN pad or text size picker, then hide
  // the on-screen keyboard, then step back a screen. Done → Home; the staff menu
  // signs out. Ctrl+Alt+S opens staff sign-in (same as the Staff button).
  function onKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.altKey && e.key.toLowerCase() === 's') {
      e.preventDefault();
      flow.textSizeOpen = false;
      flow.staffSignInOpen = !flow.staffToken;
      return;
    }
    if (e.key !== 'Escape') return;
    e.preventDefault();
    if (flow.staffSignInOpen) {
      flow.staffSignInOpen = false;
    } else if (flow.textSizeOpen) {
      flow.textSizeOpen = false;
    } else if (flow.keyboard) {
      (document.activeElement as HTMLElement | null)?.blur();
      flow.keyboard = null;
    } else if (flow.screen === 'done' || flow.screen === 'staff') {
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
    flow.keyboard = null;
    document.querySelector('.kiosk main')?.scrollTo(0, 0);
  });
</script>

<svelte:window onkeydown={onKeydown} onfocusin={onFocusIn} onfocusout={onFocusOut} onpointerdown={onPointerDown} />

<div class="kiosk">
  {#if flow.screen === 'home'}<Home />
  {:else if flow.screen === 'service'}<ServiceStep />
  {:else if flow.screen === 'time'}<TimeStep />
  {:else if flow.screen === 'calendar'}<CalendarStep />
  {:else if flow.screen === 'details'}<DetailsStep />
  {:else if flow.screen === 'done'}<DoneStep />
  {:else if flow.screen === 'staff'}<StaffMenu />
  {:else if flow.screen === 'staffClients'}<StaffClients />
  {:else if flow.screen === 'staffClient'}<StaffClient />
  {/if}
  <Keyboard />
  {#if flow.staffSignInOpen}<StaffSignIn />{/if}
</div>
