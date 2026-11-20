# Healing Hideaway POC: Progress

Tracks where the in-office kiosk POC stands. The build steps are in [`healing-hideaway-kiosk-poc-guide.md`](../healing-hideaway-kiosk-poc-guide.md).

**Current phase:** 2. App setup (next: install prerequisites)

---

## Phases

Each phase ends with something runnable or testable, and its own commit.

### Phase 1: Booking logic
Plain Rust crate (`kiosk-core/`) in a Cargo workspace with the existing `pos` crate. No Tauri yet.
- [x] Workspace set up; `pos` still builds and tests
- [x] Domain types
- [x] Availability: 30-minute grid, 24-hour notice, lunch, 15-minute buffer, closed days, month view
- [x] In-memory store: services, bookings, create booking (backend re-check), mark paid
- [x] **Done when:** `cargo test` passes (19 tests in `kiosk-core`, 5 in `pos`; clippy clean)

### Phase 2: App setup
- [ ] Install prerequisites (WebKit and other system libraries, Node/npm, `cage`): needs sudo, run by Jason
- [ ] Create the Tauri 2 + Svelte app in this repo, depending on `kiosk-core`
- [ ] Tauri commands wired up (thin wrappers: they read the clock, lock the store, call `kiosk-core`, map errors to strings)
- [ ] **Done when:** `npm run tauri dev` opens a window that lists the services

### Phase 3: App shell
- [ ] Design tokens, 1080p scaling, kiosk hardening CSS
- [ ] Screen router, TopBar, TextSizePicker, idle reset, Home
- [ ] **Done when:** you can click around empty screens, change text size, and it resets after 90 s

### Phase 4: Booking flow
- [ ] Service, Time (week strip), Calendar
- [ ] Details with on-screen keyboard and phone number pad
- [ ] Done screen
- [ ] **Done when:** a booking works with touch, mouse or keyboard, and booking the same slot twice is rejected

### Phase 5: Acceptance pass
- [ ] Every item in the guide's acceptance checklist (section 9) passes

### Phase 6: Kiosk deployment
- [ ] `.rpm` build, `cage`, systemd service on the office machine
- [ ] **Done when:** it boots straight into the app and nothing else can be reached

---

## Decisions

| Date | Decision |
|---|---|
| 2026-11-19 | Payment is taken on Lindsey's mobile device, not in the app. |
| 2026-11-19 | Bookings are confirmed immediately and start as **Unpaid**; staff mark them paid by hand. |
| 2026-11-19 | Service lengths are 30, 60 and 90 minutes; start times on a 30-minute grid. |
| 2026-11-19 | 24-hour minimum notice (no same-day bookings). |
| 2026-11-19 | Post-POC database: Microsoft SQL Server via `tiberius` (see `README.md`). |
| 2026-11-19 | `cage` without `-s` in the boot service; on-screen keyboard gets `- _ + '`, a phone number pad, and Esc handling. |
| 2026-11-20 | Build in this repo, in phases. Kiosk logic lives in its own crate next to `pos`. |
| 2026-11-20 | `kiosk-core` holds all booking rules, including `create_booking` (the guide had them in `commands.rs`). It never reads the clock: callers pass `now`. Errors are a `BookingError` enum whose messages are client-facing. Where the guide and the code differ, the code wins. |

## Open questions

- **Gift cards:** how they're sold, issued and redeemed. Not in the POC until decided.
- **Real menu:** service names, which lengths each offers, and prices (placeholders are $0).
- **Real hours:** Lindsey's weekly hours and lunch break (placeholders in `Schedule::healing_hideaway_default`).
- **Required client details:** placeholder rule is name plus phone or email.

## Log

- **2026-11-19:** Reviewed and revised the POC guide; committed it (`91512e6`).
- **2026-11-20:** Agreed on the phase plan. Phase 1 done: Cargo workspace (`pos` + `kiosk-core`), domain types, availability, in-memory store; 19 tests.
