<!-- Confirming here creates the booking immediately (as Unpaid) and goes to Done.
     There is no Pay step: payment is taken at the appointment. -->
<script lang="ts">
  import { api, type ClientDetails } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import { longDate } from '$lib/dates';
  import TopBar from '$lib/components/TopBar.svelte';
  import StepFooter from '$lib/components/StepFooter.svelte';

  const fields: { key: keyof ClientDetails; label: string; type: string }[] = [
    { key: 'name', label: 'Name', type: 'text' },
    { key: 'phone', label: 'Phone', type: 'tel' },
    { key: 'email', label: 'Email', type: 'email' },
  ];

  let error = $state('');
  let slotTaken = $state(false);
  let saving = $state(false);

  const name = $derived(flow.client.name.trim());
  const phone = $derived(flow.client.phone.trim());
  const email = $derived(flow.client.email.trim());
  const emailOk = $derived(email === '' || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email));
  // PLACEHOLDER rule: name plus a phone or email. Adjust to what Lindsey needs.
  const ready = $derived(name !== '' && (phone !== '' || email !== '') && emailOk);
  const hint = $derived(
    !name ? 'Add your name' : !phone && !email ? 'Add a phone or email' : !emailOk ? 'Check the email address' : '',
  );

  async function confirm() {
    if (saving || !ready || flow.serviceId === null) return;
    saving = true;
    error = '';
    slotTaken = false;
    try {
      flow.confirmation = await api.createBooking(flow.serviceId, flow.minutes, flow.date, flow.start, {
        name,
        phone,
        email,
      });
      flow.screen = 'done';
    } catch (e) {
      error = String(e);
      slotTaken = error.startsWith('That time was just taken');
    } finally {
      saving = false;
    }
  }

  function pickAnotherTime() {
    flow.start = '';
    flow.startLabel = '';
    flow.screen = 'time';
  }
</script>

<TopBar step={3} label="Your details" />
<main>
  <h1 style="font-size:2.4rem">Your details</h1>
  <p class="muted" style="margin-top:0.4rem">
    {longDate(flow.date)} at {flow.startLabel} · {flow.minutes} min. Payment is taken at your appointment.
  </p>

  <div class="fields">
    <!-- autocomplete off: a shared kiosk must never suggest the previous client's details -->
    {#each fields as f (f.key)}
      <label for="field-{f.key}">{f.label}</label>
      <input
        id="field-{f.key}"
        type={f.type}
        autocomplete="off"
        spellcheck="false"
        bind:value={flow.client[f.key]}
        aria-invalid={f.key === 'email' && !emailOk}
      />
    {/each}
  </div>

  {#if error}
    <div class="error" role="alert">
      <p>{error}</p>
      {#if slotTaken}<button onclick={pickAnotherTime}>Pick another time</button>{/if}
    </div>
  {/if}
</main>
<StepFooter nextLabel={saving ? 'Booking…' : 'Confirm booking'} disabled={!ready || saving} {hint} onNext={confirm} />

<style>
  .fields {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    align-items: center;
    gap: 0.8rem 1.2rem;
    margin-top: 1.2rem;
    max-width: 44rem;
  }
  :global(html[data-text-size='large']) .fields { grid-template-columns: minmax(0, 1fr); gap: 0.4rem; }
  label { font-weight: 700; }
  .error {
    margin-top: 1.2rem;
    padding: 1rem 1.2rem;
    border: 1px solid var(--accent);
    border-radius: 0.9rem;
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }
</style>
