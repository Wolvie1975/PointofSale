# Healing Hideaway POC: Progress

Tracks where the in-office kiosk POC stands. The build steps are in [`healing-hideaway-kiosk-poc-guide.md`](../healing-hideaway-kiosk-poc-guide.md).

**Current phase:** 7. Staff area (Jason to try it); 6. office machine install still open

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
- [x] Install prerequisites (WebKit and other system libraries, Node 22/npm 10, `cage`)
- [x] Create the Tauri 2 + Svelte app in `kiosk/`, depending on `kiosk-core`; `kiosk/src-tauri` is a workspace member
- [x] Tauri commands wired up (thin wrappers: they read the clock, lock the store, call `kiosk-core`, map errors to strings)
- [x] Typed `src/lib/api.ts`; temporary page lists the services; fonts installed
- [x] **Done when:** `npm run tauri dev` opens a window that lists the services (confirmed by Jason)

### Phase 3: App shell
- [x] Design tokens, 1080p scaling, kiosk hardening CSS
- [x] Screen router (with Esc), TopBar, TextSizePicker, idle reset, Home
- [x] Placeholder screens for every booking step, so the flow can be clicked through
- [x] **Done when:** you can click around empty screens, change text size, and it resets after 90 s (confirmed by Jason)

### Phase 4: Booking flow
- [x] Service, Time (week strip), Calendar
- [x] Details with on-screen keyboard and phone number pad
- [x] Done screen
- [x] **Done when:** a booking works with touch, mouse or keyboard, and booking the same slot twice is rejected (confirmed by Jason; double booking covered by `kiosk-core` tests)

### Phase 5: Acceptance pass
- [x] Every item in the guide's acceptance checklist (section 9) passes, except #1 and touch (#4), which move to Phase 6 on real hardware

| # | Check | Status |
|---|---|---|
| 1 | Full-screen 1920×1080, no chrome, no desktop; Ctrl+Alt+F2–F6 does nothing | Deferred to Phase 6 (needs the kiosk config and `cage`) |
| 2 | No horizontal scrolling at Small/Medium/Large | ✓ Confirmed by Jason |
| 3 | Large text: grids drop to 2 columns; long content scrolls vertically | ✓ Confirmed by Jason |
| 4 | Touch only: finish a booking | Moved to Phase 6 (needs the touchscreen) |
| 5 | Mouse only: hover feedback, no right-click menu | ✓ Confirmed by Jason |
| 6 | Keyboard only: Tab/Enter reach everything, focus outline visible; Esc order | ✓ Confirmed by Jason |
| 7 | On-screen keyboard incl. `- _ + '`, phone number pad, Previous/Next/Hide | Confirmed in Phase 4 |
| 8 | Closed days (Tue, Thu, Fri) Closed; past days disabled | ✓ tests + confirmed in Phase 4 |
| 9 | Unavailable shows no names/details in UI or responses | ✓ test `ui_responses_reveal_no_client_details` |
| 10 | Same slot twice rejected by backend | ✓ test `same_slot_cannot_be_booked_twice` |
| 11 | Confirm → Done right away with summary and payment note | Confirmed in Phase 4 |
| 12 | New booking Unpaid; mark paid works and is idempotent | ✓ tests |
| 13 | 30-minute grid; only 30/60/90 lengths | ✓ tests |
| 14 | Idle 90 s → Home, everything cleared | Confirmed in Phase 3 |
| 15 | Text size resets to Medium for the next client | Confirmed in Phase 3 |
| 16 | No payment or card fields anywhere | ✓ code search of `kiosk/src` |

### Phase 6: Kiosk deployment
- [x] Kiosk window config (`kiosk/src-tauri/tauri.kiosk.conf.json`) and `npm run build:kiosk`
- [x] `.rpm` builds (2 MB; installs `/usr/bin/healing-hideaway-kiosk`; requires `cage`, GTK, WebKit)
- [x] `kiosk/deploy/kiosk.service` and `kiosk/deploy/README.md` (install, SSH-first lockdown, checks, maintenance, undo)
- [x] RPM installed and working on Jason's Fedora machine
- [x] Staff-only "close app" with PIN (Jason asked for it after testing)
- [ ] Install on the office machine and run the checks in `kiosk/deploy/README.md` (Jason; needs sudo there)
- [ ] **Done when:** it boots straight into the app and nothing else can be reached

### Phase 7: Staff area (client notes)
Mocked in memory, like bookings; SQL Server replaces it after the POC.
- [x] `kiosk-core`: clients built from bookings (matched by phone or email), private notes (add / edit / delete), client list and client record, sample data
- [x] Staff sign-in: correct PIN gives a session token; every staff command needs it; ends after 5 min without staff activity, on Sign out, or on the 90 s idle reset
- [x] `mark_booking_paid` replaced by `staff_mark_paid`, which needs staff sign-in (closes the gap flagged in the guide)
- [x] Visible **Staff** button in the top bar (Jason: no hidden options in the POC); staff menu with Clients and Close app
- [x] Clients list with search; client page with private notes and bookings (Mark paid with confirm)
- [x] On-screen keyboard reworked to type into any text field at the cursor (search and notes included)
- [x] Tests: 39 in `kiosk-core`, including "client-facing responses never include notes"
- [ ] **Done when:** Jason tries it in `npm run tauri dev` (PIN `0000`)

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
| 2026-11-20 | App lives in `kiosk/`; binary is `healing-hideaway-kiosk` (`mainBinaryName`), product name "Healing Hideaway". Template's opener plugin removed (a kiosk must not open outside links). Bundle target is `.rpm` only. Release profile moved to the workspace root. |
| 2026-11-20 | Shell details: UI scale uses the smaller of width/1422 and height/800, so a narrow window never scrolls sideways. Esc is handled only in the router (closes text size picker → hides keyboard → goes back). Back/Esc targets live in `backFrom` in `flow.svelte.ts`. Home has only "Book a massage" until gift cards are decided; staff sign-in comes with the console. |
| 2026-11-20 | Booking flow details: the week strip marks Closed/Full days and defaults to the first day with a time; the calendar shows "Few left" (not "2 left", since it can be 1). The on-screen keyboard is sized by screen scale, not text size, so Large text doesn't make it fill the screen; its Continue says Done on the last field. Moving fields on the on-screen keyboard also moves real focus. Inputs have autocomplete off so the kiosk never suggests a previous client's details. Light email format check. |
| 2026-11-20 | One shared look for every selectable option (`.choice` / `.choice.on` in `app.css`): lengths, week days, calendar days, times, text size. The selected option is accent-filled with a ✓ and a glow (Jason asked for clearer selection). |
| 2026-11-20 | Kiosk window settings live in a separate `tauri.kiosk.conf.json` merged at build time (`npm run build:kiosk`), so `tauri dev` keeps a normal window. The RPM declares `cage` as a dependency. Set up SSH before enabling the kiosk service, since the keyboard can't get out afterwards. |
| 2026-11-20 | Staff close: hold "Healing Hideaway" in the top bar 3 s (or Ctrl+Alt+S) → PIN pad → Close app. PIN checked in Rust (`kiosk-core/src/staff.rs`), 5 wrong tries lock it for 60 s. PIN comes from `HH_STAFF_PIN` (4–8 digits): a root-only systemd override on the office machine; dev builds fall back to `0000`; release builds with no PIN can't be closed from the screen. Under the kiosk service, closing restarts the app fresh. |
| 2026-11-20 | Release RPMs are committed to git in `releases/` (Jason's call, so they're on GitHub). `npm run release:kiosk` builds and copies `healing-hideaway-kiosk-<version>-1.x86_64.rpm` there. Tauri reads its version from `kiosk/package.json`; bump it for each kept release. |
| 2026-11-20 | Staff area replaces the hidden close: visible **Staff** button (still PIN-protected) → menu with Clients and Close app. Staff screens have a tinted top bar so it's obvious nobody is in the client flow. Clients page: notes + bookings with Mark paid; notes can be added, edited and deleted (Jason's picks). |
| 2026-11-20 | Clients are one record per person, matched by phone (7+ digits) or email (any case); a later booking updates their details. Bookings now store `client_id` instead of a copy of the details. `BookingError` renamed `StoreError`. |
| 2026-11-20 | The app starts with three obviously fake sample clients (past visits, notes) so the staff area has data. POC only; remove before real use. |
| 2026-11-20 | Minimum Rust raised from 1.85 to 1.90 across the workspace. Tauri 2.12 needs it, and the Rust `tauri` crate must match the npm `@tauri-apps/api` major/minor version (Cargo's resolver had picked 2.11 to fit 1.85). |
| 2026-11-20 | `kiosk-core` holds all booking rules, including `create_booking` (the guide had them in `commands.rs`). It never reads the clock: callers pass `now`. Errors are a `BookingError` enum whose messages are client-facing. Where the guide and the code differ, the code wins. |

## Open questions

- **Gift cards:** how they're sold, issued and redeemed. Not in the POC until decided.
- **Real menu:** service names, which lengths each offers, and prices (placeholders are $0).
- **Real hours:** Lindsey's weekly hours and lunch break (placeholders in `Schedule::healing_hideaway_default`).
- **Required client details:** placeholder rule is name plus phone or email.
- **npm audit (2026-11-20):** 3 findings in SvelteKit 2 server features (form actions, remote functions, cookies, server rendering). Not used by this static, serverless app; the only fix is SvelteKit 3 (breaking). Revisit before deployment.

## Log

- **2026-11-19:** Reviewed and revised the POC guide; committed it (`91512e6`).
- **2026-11-20:** Agreed on the phase plan. Phase 1 done: Cargo workspace (`pos` + `kiosk-core`), domain types, availability, in-memory store; 19 tests (`910ca46`).
- **2026-11-20:** Phase 2: prerequisites installed; Tauri app created in `kiosk/` and wired to `kiosk-core`. Builds; tests, clippy and `svelte-check` clean. Fixed a Tauri Rust/npm version mismatch (Rust 1.90 minimum). Window confirmed listing the services. Phase 2 done (`6f46750`).
- **2026-11-20:** Phase 3 built: tokens, scaling, router, TopBar, text size picker, idle reset, Home, placeholder steps. `svelte-check` clean. Confirmed by Jason. Phase 3 done (`92076df`).
- **2026-11-20:** Phase 4 built: Service, Time, Calendar, Details (keyboard + number pad), Done, all calling `kiosk-core`. `svelte-check` clean. Confirmed by Jason. Phase 4 done (`02d229a`).
- **2026-11-20:** Phase 5 started: checklist reviewed against code and tests; fixed Tab escaping the text size panel; made selected options much more obvious, with one shared style (`fd1903f`).
- **2026-11-20:** Phase 6 prepared: kiosk build config, RPM builds, systemd unit and deploy guide written. Waiting on install on the office machine.
- **2026-11-20:** Phase 5 done: Jason confirmed the on-screen checks.
- **2026-11-20:** Phase 6 files committed (`f4be218`). Jason installed the RPM on his Fedora machine; it works. Added staff-only close with PIN (7 new tests, 26 in `kiosk-core`) (`275807d`). Added `releases/` and `npm run release:kiosk` (`ec1e621`).
- **2026-11-20:** Phase 7 built: staff area with client notes and bookings, mocked in memory. 39 `kiosk-core` tests; clippy and `svelte-check` clean. Waiting on Jason's try-out.
