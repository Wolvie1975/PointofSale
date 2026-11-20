<script lang="ts">
  import { api, type DayAvailability, type DayStatus } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import { daysFromToday, fromIso, longDate, monthTitle } from '$lib/dates';
  import TopBar from '$lib/components/TopBar.svelte';
  import StepFooter from '$lib/components/StepFooter.svelte';
  import SlotPicker from '$lib/components/SlotPicker.svelte';

  // Start on the month of the chosen day, or of tomorrow
  const first = fromIso(flow.date || daysFromToday(1));
  const earliest = fromIso(daysFromToday(1));
  let year = $state(first.getFullYear());
  let month = $state(first.getMonth() + 1); // 1–12

  let days = $state<DayAvailability[] | null>(null);
  let request = 0;

  $effect(() => {
    const [y, m, minutes] = [year, month, flow.minutes];
    const mine = ++request;
    days = null;
    api.monthDays(y, m, minutes).then((d) => {
      if (mine === request) days = d;
    });
  });

  const atEarliest = $derived(year === earliest.getFullYear() && month === earliest.getMonth() + 1);
  // Blank cells before the 1st, so weeks start on Sunday
  const lead = $derived(new Date(year, month - 1, 1).getDay());

  function shift(by: number) {
    const d = new Date(year, month - 1 + by, 1);
    year = d.getFullYear();
    month = d.getMonth() + 1;
  }

  function chooseDay(iso: string) {
    if (iso === flow.date) return;
    flow.date = iso;
    flow.start = '';
    flow.startLabel = '';
  }

  const bookable = (s: DayStatus) => s === 'open' || s === 'limited';
  const note: Record<DayStatus, string> = { past: '', closed: 'Closed', open: '', limited: 'Few left', full: 'Full' };
</script>

<TopBar step={2} label="Time" />
<main>
  <h1 style="font-size:2.4rem">Pick a day</h1>

  <div class="layout">
    <section aria-label="Calendar">
      <div class="month-nav">
        <button onclick={() => shift(-1)} disabled={atEarliest} aria-label="Previous month">‹</button>
        <h2 aria-live="polite">{monthTitle(year, month)}</h2>
        <button onclick={() => shift(1)} aria-label="Next month">›</button>
      </div>

      <div class="grid">
        {#each ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as w (w)}
          <span class="dow muted" aria-hidden="true">{w}</span>
        {/each}
        {#if days}
          {#each Array(lead) as _, i (i)}<span></span>{/each}
          {#each days as d (d.date)}
            {@const on = flow.date === d.date}
            <button
              class="day {d.status}"
              class:choice={bookable(d.status)}
              class:on
              aria-pressed={on}
              disabled={!bookable(d.status)}
              aria-label="{longDate(d.date)}{note[d.status] ? `, ${note[d.status]}` : ''}"
              onclick={() => chooseDay(d.date)}
            >
              <span>{fromIso(d.date).getDate()}</span>
              <small class="sub">{on ? '✓' : note[d.status]}</small>
            </button>
          {/each}
        {/if}
      </div>
    </section>

    <section aria-label="Times">
      {#if flow.date}<h2 class="day-title">{longDate(flow.date)}</h2>{/if}
      <SlotPicker />
    </section>
  </div>
</main>
<StepFooter
  nextLabel="Continue to your details"
  disabled={!flow.start}
  hint={flow.start ? `${longDate(flow.date)} at ${flow.startLabel}` : 'Pick a day, then a time'}
  onNext={() => (flow.screen = 'details')}
/>

<style>
  .layout { display: grid; grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr); gap: 2rem; margin-top: 1.2rem; }
  :global(html[data-text-size='large']) .layout { grid-template-columns: minmax(0, 1fr); }
  .month-nav { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.8rem; }
  .month-nav h2 { font-size: 1.5rem; }
  .grid { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); gap: 0.35rem; }
  .dow { text-align: center; font-size: 0.85rem; padding-bottom: 0.2rem; }
  .day {
    min-height: 3.5rem;
    padding: 0.2rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    line-height: 1.1;
  }
  .day small { font-size: 0.7rem; min-height: 1em; }
  .day.past, .day.closed { background: transparent; border-color: transparent; color: var(--dim); }
  .day.full { background: transparent; border-style: dashed; color: var(--dim); }
  .day.limited:not(.on) small { color: var(--accent); }
  .day-title { font-size: 1.5rem; margin-bottom: 0.8rem; }
</style>
