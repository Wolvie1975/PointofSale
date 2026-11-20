<!-- On-screen keyboard. Types into whichever text field has focus (any screen),
     at the cursor. Phone fields get a number pad; notes (textareas) get a New line key.
     Shown while flow.keyboard is set (the router sets it when a text field gets focus).
     Sized in px × --ui-scale, not rem, so Large text doesn't make it swallow the screen. -->
<script lang="ts">
  import { flow } from '$lib/flow.svelte';

  type Field = HTMLInputElement | HTMLTextAreaElement;

  let shift = $state(false);
  let label = $state('');
  let isTextarea = $state(false);
  let isLast = $state(false);

  const rows = ['1234567890', 'qwertyuiop', 'asdfghjkl', 'zxcvbnm'].map((r) => r.split(''));
  // Needed for emails (first_last@, name+tag@, hyphenated domains) and names (O'Brien, Mary-Kate)
  const symbols = ['-', '_', '+', "'"];
  const pad = [['1', '2', '3'], ['4', '5', '6'], ['7', '8', '9'], ['+', '0', '-']];

  function current(): Field | null {
    const el = document.activeElement;
    return el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement ? el : null;
  }

  /** Text fields on the current screen, in page order (for Previous / Next). */
  function fields(): Field[] {
    return [...document.querySelectorAll<Field>('.kiosk main input:not([disabled]), .kiosk main textarea:not([disabled])')];
  }

  /** Refresh the header and keys for the focused field. */
  function sync() {
    const el = current();
    if (!el) return;
    label = el.labels?.[0]?.textContent?.trim() || el.getAttribute('aria-label') || el.placeholder || 'Text';
    isTextarea = el instanceof HTMLTextAreaElement;
    const all = fields();
    isLast = all.indexOf(el) === all.length - 1;
  }

  $effect(() => {
    if (flow.keyboard) sync();
    const onFocus = () => sync();
    window.addEventListener('focusin', onFocus);
    return () => window.removeEventListener('focusin', onFocus);
  });

  /** Tell Svelte's bind:value about the change. */
  const changed = (el: Field) => el.dispatchEvent(new Event('input', { bubbles: true }));

  function type(text: string) {
    const el = current();
    if (!el) return;
    const ch = shift ? text.toUpperCase() : text;
    shift = false;
    // Email inputs have no cursor position to insert at; append instead
    if (el.selectionStart === null || el.selectionEnd === null) el.value += ch;
    else el.setRangeText(ch, el.selectionStart, el.selectionEnd, 'end');
    changed(el);
  }

  function backspace() {
    const el = current();
    if (!el) return;
    const [start, end] = [el.selectionStart, el.selectionEnd];
    if (start === null || end === null) el.value = el.value.slice(0, -1);
    else if (start !== end) el.setRangeText('', start, end, 'end');
    else if (start > 0) el.setRangeText('', start - 1, start, 'end');
    changed(el);
  }

  function move(step: number) {
    const all = fields();
    const el = current();
    if (!all.length) return;
    const i = el ? all.indexOf(el) : -1;
    all[(i + step + all.length) % all.length].focus();
  }

  const hide = () => current()?.blur(); // the router hides the keyboard when focus leaves

  // Keep focus in the field while tapping keys
  const keep = (e: PointerEvent) => e.preventDefault();
</script>

{#if flow.keyboard}
  <div class="kb" role="group" aria-label="On-screen keyboard">
    <div class="kb-head">
      <span>Typing in: {label}</span>
      <span class="kb-nav">
        <button onpointerdown={keep} onclick={() => move(-1)}>‹ Previous</button>
        <button onpointerdown={keep} onclick={() => move(1)}>Next field ›</button>
        <button onpointerdown={keep} onclick={hide} aria-label="Hide keyboard">Hide ⌄</button>
      </span>
    </div>

    {#if flow.keyboard === 'phone'}
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
          <button class="primary" onpointerdown={keep} onclick={() => (isLast ? hide() : move(1))}>
            {isLast ? 'Done' : 'Continue'}
          </button>
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
        {#if isTextarea}
          <button onpointerdown={keep} onclick={() => type(',')}>,</button>
          <button onpointerdown={keep} onclick={() => type('.')}>.</button>
          <button style="flex:1.6" onpointerdown={keep} onclick={() => type('\n')}>↵ New line</button>
        {:else}
          <button onpointerdown={keep} onclick={() => type('.')}>.</button>
          <button onpointerdown={keep} onclick={() => type('.com')}>.com</button>
        {/if}
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
