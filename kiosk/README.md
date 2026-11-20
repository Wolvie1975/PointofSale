# Healing Hideaway kiosk

The in-office booking app: Tauri 2 (Rust) with a SvelteKit + TypeScript UI. Booking rules live in [`../kiosk-core`](../kiosk-core); this app only wires them to the screen.

- Build guide: [`../healing-hideaway-kiosk-poc-guide.md`](../healing-hideaway-kiosk-poc-guide.md)
- Progress: [`../claude/healing-hideaway-pos-progress.md`](../claude/healing-hideaway-pos-progress.md)

## Run

```bash
npm install
npm run tauri dev     # dev window with hot reload
npm run check         # Svelte/TypeScript check
npm run release:kiosk # kiosk .rpm, copied to ../releases/ (see deploy/README.md)
```

If the window comes up blank, prefix the command with `WEBKIT_DISABLE_DMABUF_RENDERER=1`.

The Rust `tauri` crate and the npm `@tauri-apps/api` package must stay on the same major/minor version.

## Recommended IDE setup

VS Code with the Svelte, Tauri and rust-analyzer extensions.
