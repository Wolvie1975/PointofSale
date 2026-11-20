<script lang="ts">
  import '@fontsource/dm-sans/400.css';
  import '@fontsource/dm-sans/700.css';
  import '@fontsource/dm-serif-display/400.css';
  import '../app.css';
  import { onMount } from 'svelte';

  let { children } = $props();

  onMount(() => {
    // The UI is designed at 1422×800 logical px and scaled to fit the window.
    // Using the smaller ratio means a narrower window never scrolls sideways.
    const setScale = () => {
      const scale = Math.min(window.innerWidth / 1422, window.innerHeight / 800);
      document.documentElement.style.setProperty('--ui-scale', String(scale));
    };
    setScale();
    window.addEventListener('resize', setScale);

    // Kiosk hardening: no long-press / right-click menu
    const noMenu = (e: Event) => e.preventDefault();
    document.addEventListener('contextmenu', noMenu);

    return () => {
      window.removeEventListener('resize', setScale);
      document.removeEventListener('contextmenu', noMenu);
    };
  });
</script>

{@render children()}
