<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import { daysFromToday, fromIso, longDate, shortWeekday } from '$lib/dates';
  import TopBar from '$lib/components/TopBar.svelte';
  import StepFooter from '$lib/components/StepFooter.svelte';
  import SlotPicker from '$lib/components/SlotPicker.svelte';

  type DayState = 'loading' | 'closed' | 'full' | 'open';

  // Next 7 days starting tomorrow (24 h notice means today is never bookable)
  const days = Array.from({ length: 7 }, (_, i) => daysFromToday(i + 1));
  let dayState = $state<Record<string, DayState>>(Object.fromEntries(days.map((d) => [d, 'loading'])));

  onMount(async () => {
    const results = await Promise.all(days.map((d) => api.daySlots(d, flow.minutes).catch(() => [])));
    results.forEach((slots, i) => {
      dayState[days[i]] =
        slots.length === 0 ? 'closed' : slots.some((s) => s.status === 'open') ? 'open' : 'full';
    });
    // Default to the first day that still has a time
    if (!flow.date) flow.date = days.find((d) => dayState[d] === 'open') ?? days[0];
  });

  function chooseDay(iso: string) {
    if (iso === flow.date) return;
    flow.date = iso;
    flow.start = '';
    flow.startLabel = '';
  }

  const note: Record<DayState, string> = { loading: '', closed: 'Closed', full: 'Full', open: '' };
</script>

<TopBar step={2} label="Time" />
<main>
  <div class="head">
    <h1 style="font-size:2.4rem">When works for you?</h1>
    <button onclick={() => (flow.screen = 'calendar')}>See full calendar ›</button>
  </div>

  <div class="week" role="group" aria-label="Next 7 days">
    {#each days as d (d)}
      {@const on = flow.date === d}
      <button
        class="choice"
        class:on
        aria-pressed={on}
        disabled={dayState[d] === 'closed'}
        onclick={() => chooseDay(d)}
      >
        {shortWeekday(d)}
        <span class="display num">{fromIso(d).getDate()}</span>
        <small class="sub">{on ? '✓' : note[dayState[d]]}</small>
      </button>
    {/each}
  </div>

  {#if flow.date}
    <h2 class="day-title">{longDate(flow.date)}</h2>
  {/if}
  <SlotPicker />
</main>
<StepFooter
  nextLabel="Continue to your details"
  disabled={!flow.start}
  hint={flow.start ? `${longDate(flow.date)} at ${flow.startLabel}` : 'Pick a time'}
  onNext={() => (flow.screen = 'details')}
/>

<style>
  .head { display: flex; justify-content: space-between; align-items: center; gap: 1rem; flex-wrap: wrap; }
  .week { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); gap: 0.6rem; margin: 1.2rem 0; }
  .week button { display: flex; flex-direction: column; align-items: center; padding: 0.5rem 0.2rem; line-height: 1.15; }
  .num { font-size: 1.7rem; }
  .week small { min-height: 1em; }
  .day-title { font-size: 1.5rem; margin-bottom: 0.8rem; }
</style>
