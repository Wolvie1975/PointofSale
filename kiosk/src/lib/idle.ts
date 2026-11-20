import { flow, resetFlow } from './flow.svelte';

/** Returns to Home, with everything cleared, after `ms` without a touch, click or key. */
export function startIdleTimer(ms = 90_000) {
  let timer: ReturnType<typeof setTimeout>;
  const events = ['pointerdown', 'keydown', 'wheel'] as const;

  const reset = () => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (flow.screen !== 'home' || flow.textSize !== 'medium' || flow.textSizeOpen || flow.staffOpen) resetFlow();
    }, ms);
  };

  events.forEach((e) => window.addEventListener(e, reset, { passive: true }));
  reset();
  return () => {
    clearTimeout(timer);
    events.forEach((e) => window.removeEventListener(e, reset));
  };
}
