<!-- Staff only: enter the PIN to close the app. Opened by holding the name in the
     top bar for 3 s, or Ctrl+Alt+S. The PIN is checked in Rust, never here. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { flow } from '$lib/flow.svelte';

  const MAX = 8;
  let pin = $state('');
  let message = $state('');
  let checking = $state(false);
  let configured = $state(true);

  onMount(async () => {
    configured = await api.staffPinConfigured().catch(() => false);
  });

  const close = () => (flow.staffOpen = false);

  function press(d: string) {
    if (pin.length < MAX) pin += d;
    message = '';
  }
  const backspace = () => (pin = pin.slice(0, -1));

  async function submit() {
    if (checking || pin.length < 4) return;
    checking = true;
    try {
      await api.staffCloseApp(pin); // the app exits on success
    } catch (e) {
      message = String(e);
      pin = '';
    } finally {
      checking = false;
    }
  }

  // A physical keyboard works too: digits, Backspace, Enter. (Esc is handled by the router.)
  function onKeydown(e: KeyboardEvent) {
    if (/^[0-9]$/.test(e.key)) press(e.key);
    else if (e.key === 'Backspace') backspace();
    else if (e.key === 'Enter') submit();
    else return;
    e.preventDefault();
  }

  function focusPanel(node: HTMLElement) {
    node.focus();
  }

  const keep = (e: PointerEvent) => e.preventDefault();
</script>

<svelte:window onkeydown={onKeydown} />

<button class="backdrop" aria-label="Cancel" tabindex="-1" onclick={close}></button>

<div class="panel" role="dialog" aria-modal="true" aria-label="Staff: close the app" tabindex="-1" use:focusPanel>
  <h2>Staff: close the app</h2>

  {#if !configured}
    <p class="msg">No staff PIN is set up on this machine, so the app can't be closed from here.</p>
    <button onclick={close}>OK</button>
  {:else}
    <p class="muted">Enter the staff PIN.</p>
    <div class="dots" aria-label="{pin.length} digits entered">
      {#each Array(Math.max(4, pin.length)) as _, i (i)}
        <span class:filled={i < pin.length}></span>
      {/each}
    </div>
    <p class="msg" role="alert">{message}</p>

    <div class="pad">
      {#each ['1', '2', '3', '4', '5', '6', '7', '8', '9'] as d (d)}
        <button onpointerdown={keep} onclick={() => press(d)}>{d}</button>
      {/each}
      <button onpointerdown={keep} onclick={backspace} aria-label="Backspace">⌫</button>
      <button onpointerdown={keep} onclick={() => press('0')}>0</button>
      <span></span>
    </div>

    <div class="actions">
      <button onclick={close}>Cancel</button>
      <button class="primary" disabled={pin.length < 4 || checking} onclick={submit}>Close app</button>
    </div>
  {/if}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.6);
    border: none;
    border-radius: 0;
    min-height: 0;
    padding: 0;
    cursor: default;
    z-index: 20;
  }
  .backdrop:hover { box-shadow: none; filter: none; }
  .panel {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 21;
    width: min(24rem, calc(100vw - 2rem));
    max-height: calc(100vh - 2rem);
    overflow-y: auto;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    border-radius: 1.2rem;
    padding: 1.4rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
    text-align: center;
  }
  .panel:focus { outline: none; }
  h2 { font-size: 1.5rem; }
  .dots { display: flex; gap: 0.6rem; margin-top: 0.2rem; }
  .dots span { width: 0.9rem; height: 0.9rem; border-radius: 50%; border: 2px solid var(--muted); }
  .dots span.filled { background: var(--text); border-color: var(--text); }
  .msg { min-height: 1.3em; color: var(--accent); }
  .pad { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.5rem; width: 100%; }
  .pad button { font-size: 1.3rem; font-weight: 700; }
  .actions { display: flex; gap: 0.6rem; width: 100%; }
  .actions button { flex: 1; }
</style>
