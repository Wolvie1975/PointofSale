<!-- On-screen keyboard for flow.activeField. The phone field gets a number pad.
     Sized in px × --ui-scale, not rem, so Large text doesn't make it swallow the screen. -->
<script lang="ts">
  import { flow, type Field } from '$lib/flow.svelte';

  let shift = $state(false);
  const rows = ['1234567890', 'qwertyuiop', 'asdfghjkl', 'zxcvbnm'].map((r) => r.split(''));
  // Needed for emails (first_last@, name+tag@, hyphenated domains) and names (O'Brien, Mary-Kate)
  const symbols = ['-', '_', '+', "'"];
  const pad = [['1', '2', '3'], ['4', '5', '6'], ['7', '8', '9'], ['+', '0', '-']];
  const order: Field[] = ['name', 'phone', 'email'];
  const labels: Record<Field, string> = { name: 'Name', phone: 'Phone', email: 'Email' };

  const isLast = $derived(flow.activeField === order[order.length - 1]);

  function type(ch: string) {
    const f = flow.activeField;
    if (!f) return;
    flow.client[f] += shift ? ch.toUpperCase() : ch;
    shift = false;
  }
  function backspace() {
    const f = flow.activeField;
    if (f) flow.client[f] = flow.client[f].slice(0, -1);
  }
  function move(step: number) {
    const i = order.indexOf(flow.activeField ?? 'name');
    flow.activeField = order[(i + step + order.length) % order.length];
  }
  function hide() {
    flow.activeField = null;
    (document.activeElement as HTMLElement | null)?.blur();
  }
  // Keep focus in the input while tapping keys
  const keep = (e: PointerEvent) => e.preventDefault();
</script>

{#if flow.activeField}
  <div class="kb" role="group" aria-label="On-screen keyboard">
    <div class="kb-head">
      <span>Typing in: {labels[flow.activeField]}</span>
      <span class="kb-nav">
        <button onpointerdown={keep} onclick={() => move(-1)}>‹ Previous</button>
        <button onpointerdown={keep} onclick={() => move(1)}>Next field ›</button>
        <button onpointerdown={keep} onclick={hide} aria-label="Hide keyboard">Hide ⌄</button>
      </span>
    </div>

    {#if flow.activeField === 'phone'}
      <div class="pad">
        {#each pad as row, r (r)}
          <div class="kb-row">
            {#each row as ch (ch)}
              <button onpointerdown={keep} onclick={() => type(ch)}>{ch}</button>
            {/each}
          </div>
        {/each}
        <div class="kb-row">
          <button class="mod" onpointerdown={keep} onclick={backspace} aria-label="Backspace">⌫</button>
          <button class="primary" onpointerdown={keep} onclick={() => move(1)}>Continue</button>
        </div>
      </div>
    {:else}
      {#each rows as row, i (i)}
        <div class="kb-row">
          {#if i === 3}<button class="mod" onpointerdown={keep} aria-pressed={shift} onclick={() => (shift = !shift)} aria-label="Shift">⇧</button>{/if}
          {#each row as ch (ch)}
            <button onpointerdown={keep} onclick={() => type(ch)}>{shift ? ch.toUpperCase() : ch}</button>
          {/each}
          {#if i === 3}<button class="mod" onpointerdown={keep} onclick={backspace} aria-label="Backspace">⌫</button>{/if}
        </div>
      {/each}
      <div class="kb-row">
        <button onpointerdown={keep} onclick={() => type('@')}>@</button>
        {#each symbols as ch (ch)}
          <button onpointerdown={keep} onclick={() => type(ch)}>{ch}</button>
        {/each}
        <button style="flex:4" onpointerdown={keep} onclick={() => type(' ')} aria-label="Space">space</button>
        <button onpointerdown={keep} onclick={() => type('.')}>.</button>
        <button onpointerdown={keep} onclick={() => type('.com')}>.com</button>
        <button class="primary" style="flex:1.8" onpointerdown={keep} onclick={() => (isLast ? hide() : move(1))}>
          {isLast ? 'Done' : 'Continue'}
        </button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .kb {
    --k: var(--ui-scale);
    font-size: calc(var(--k) * 19px);
    background: #1d1b23;
    border-top: 1px solid var(--border);
    padding: calc(var(--k) * 10px) calc(var(--k) * 40px) calc(var(--k) * 14px);
    display: flex;
    flex-direction: column;
    gap: calc(var(--k) * 6px);
  }
  .kb button { min-height: calc(var(--k) * 46px); padding: 0 calc(var(--k) * 10px); border-radius: calc(var(--k) * 10px); }
  .kb-head { display: flex; justify-content: space-between; align-items: center; font-weight: 700; }
  .kb-nav { display: flex; gap: calc(var(--k) * 6px); }
  .kb-row { display: flex; gap: calc(var(--k) * 6px); }
  .kb-row button { flex: 1; background: #2e2a36; border: none; }
  .kb-row .mod { flex: 1.5; background: var(--border); }
  .kb-row button.primary { background: var(--accent); }
  .pad { width: min(100%, calc(var(--k) * 460px)); margin: 0 auto; display: flex; flex-direction: column; gap: calc(var(--k) * 6px); }
  .pad .kb-row button { font-size: 1.25em; }
  .pad .kb-row .mod { flex: 1; }
</style>
