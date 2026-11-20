<!-- Staff: one client's private notes and bookings. Clients never see this screen. -->
<script lang="ts">
  import { formatPrice, isSignedOutError, staffApi, type ClientRecord, type Note } from '$lib/api';
  import { flow, resetFlow } from '$lib/flow.svelte';
  import { shortDateTime } from '$lib/dates';
  import TopBar from '$lib/components/TopBar.svelte';

  let record = $state<ClientRecord | null>(null);
  let error = $state('');
  let draft = $state('');
  let saving = $state(false);
  let editingId = $state<string | null>(null);
  let editText = $state('');
  let confirmDeleteId = $state<string | null>(null);
  let confirmPaidId = $state<string | null>(null);

  /** Runs a staff call; a lapsed session sends staff back to Home. */
  async function run<T>(call: (token: string) => Promise<T>): Promise<T | undefined> {
    if (!flow.staffToken) {
      resetFlow();
      return;
    }
    error = '';
    try {
      return await call(flow.staffToken);
    } catch (e) {
      if (isSignedOutError(e)) resetFlow();
      else error = String(e);
    }
  }

  async function load() {
    if (!flow.staffClientId) return;
    const id = flow.staffClientId;
    const r = await run((t) => staffApi.client(t, id));
    if (r) record = r;
  }
  $effect(() => {
    load();
  });

  async function addNote() {
    if (!record || !draft.trim() || saving) return;
    saving = true;
    const id = record.client.id;
    const note = await run((t) => staffApi.addNote(t, id, draft));
    saving = false;
    if (note) {
      draft = '';
      record.notes = [note, ...record.notes];
    }
  }

  function startEdit(n: Note) {
    editingId = n.id;
    editText = n.text;
    confirmDeleteId = null;
  }

  async function saveEdit() {
    if (!record || !editingId || !editText.trim()) return;
    const id = editingId;
    const note = await run((t) => staffApi.editNote(t, id, editText));
    if (note) {
      record.notes = record.notes.map((n) => (n.id === note.id ? note : n));
      editingId = null;
    }
  }

  async function deleteNote(id: string) {
    if (!record) return;
    const ok = await run((t) => staffApi.deleteNote(t, id).then(() => true));
    if (ok) record.notes = record.notes.filter((n) => n.id !== id);
    confirmDeleteId = null;
  }

  async function markPaid(id: string) {
    const ok = await run((t) => staffApi.markPaid(t, id).then(() => true));
    confirmPaidId = null;
    if (ok) await load();
  }
</script>

<TopBar label="Client" />
<main>
  {#if record}
    {@const c = record.client}
    <h1 style="font-size:2.4rem">{c.name || 'No name'}</h1>
    <p class="muted contact">{[c.phone, c.email].filter(Boolean).join(' · ') || 'No contact details'}</p>

    {#if error}<p class="error" role="alert">{error}</p>{/if}

    <div class="layout">
      <section aria-labelledby="notes-h">
        <h2 id="notes-h">Private notes</h2>
        <p class="private">🔒 Staff only. Clients never see these.</p>

        <label for="new-note" class="sr-only">New note</label>
        <textarea id="new-note" rows="3" placeholder="Add a note about this client…" bind:value={draft}></textarea>
        <div class="row end">
          <button class="primary" disabled={!draft.trim() || saving} onclick={addNote}>Save note</button>
        </div>

        {#if record.notes.length === 0}
          <p class="muted">No notes yet.</p>
        {/if}
        <ul class="notes">
          {#each record.notes as n (n.id)}
            <li class="note">
              {#if editingId === n.id}
                <label for="edit-{n.id}" class="sr-only">Edit note</label>
                <textarea id="edit-{n.id}" rows="3" bind:value={editText}></textarea>
                <div class="row end">
                  <button onclick={() => (editingId = null)}>Cancel</button>
                  <button class="primary" disabled={!editText.trim()} onclick={saveEdit}>Save</button>
                </div>
              {:else}
                <p class="text">{n.text}</p>
                <p class="meta muted">
                  {shortDateTime(n.createdAt)}{#if n.editedAt}&nbsp;· edited {shortDateTime(n.editedAt)}{/if}
                </p>
                {#if confirmDeleteId === n.id}
                  <div class="row end" role="alert">
                    <span>Delete this note?</span>
                    <button onclick={() => (confirmDeleteId = null)}>Cancel</button>
                    <button class="primary" onclick={() => deleteNote(n.id)}>Delete</button>
                  </div>
                {:else}
                  <div class="row end">
                    <button class="small" onclick={() => startEdit(n)}>Edit</button>
                    <button class="small" onclick={() => (confirmDeleteId = n.id)}>Delete</button>
                  </div>
                {/if}
              {/if}
            </li>
          {/each}
        </ul>
      </section>

      <section aria-labelledby="bookings-h">
        <h2 id="bookings-h">Bookings</h2>
        {#if record.bookings.length === 0}
          <p class="muted">No bookings.</p>
        {/if}
        <ul class="bookings">
          {#each record.bookings as b (b.id)}
            <li class="booking">
              <div>
                <strong>{shortDateTime(b.start)}</strong>
                <span class="muted">{b.serviceName}, {b.minutes} min · {formatPrice(b.priceCents)}</span>
              </div>
              <div class="row">
                {#if b.payment === 'paid'}
                  <span class="badge paid">Paid{b.paidAt ? ` ${shortDateTime(b.paidAt)}` : ''}</span>
                {:else if confirmPaidId === b.id}
                  <span>Payment taken?</span>
                  <button class="small" onclick={() => (confirmPaidId = null)}>Cancel</button>
                  <button class="small primary" onclick={() => markPaid(b.id)}>Yes, mark paid</button>
                {:else}
                  <span class="badge unpaid">Unpaid</span>
                  <button class="small" onclick={() => (confirmPaidId = b.id)}>Mark paid</button>
                {/if}
              </div>
            </li>
          {/each}
        </ul>
      </section>
    </div>
  {:else if error}
    <p role="alert">{error}</p>
  {:else}
    <p class="muted">Loading…</p>
  {/if}
</main>
<footer>
  <button onclick={() => (flow.screen = 'staffClients')}>Back to clients</button>
</footer>

<style>
  .contact { margin-top: 0.3rem; }
  .error { margin-top: 0.8rem; color: var(--accent); }
  .layout { display: grid; grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr); gap: 2rem; margin-top: 1.4rem; }
  :global(html[data-text-size='large']) .layout { grid-template-columns: minmax(0, 1fr); }
  section { display: flex; flex-direction: column; gap: 0.7rem; }
  h2 { font-size: 1.5rem; }
  .private { font-size: 0.9rem; color: var(--accent); }
  textarea { width: 100%; }
  .row { display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
  .row.end { justify-content: flex-end; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.6rem; }
  .note, .booking {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 0.9rem;
    padding: 0.8rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .text { white-space: pre-wrap; overflow-wrap: anywhere; }
  .meta { font-size: 0.85rem; }
  .booking > div:first-child { display: flex; flex-direction: column; gap: 0.15rem; }
  button.small { min-height: 2.8rem; padding: 0 1rem; }
  .badge { padding: 0.2rem 0.7rem; border-radius: 999px; font-size: 0.85rem; font-weight: 700; }
  .badge.unpaid { background: #4a3a1e; color: #f2d29b; }
  .badge.paid { background: #1f3a2c; color: #a8e0bf; }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }
</style>
