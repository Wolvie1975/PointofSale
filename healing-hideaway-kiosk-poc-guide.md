# Healing Hideaway: In-Office App POC Guide (Rust)

A step-by-step guide for building a **proof of concept** of the in-office app: a Rust + Tauri 2 desktop app with a Svelte UI, running full-screen at 1080p on Fedora. It covers booking and the touch, mouse and keyboard rules from the design. Payment happens outside the app, on Lindsey's mobile device.

_Design reference: the Evening Retreat screens on the design canvas. Project decisions: `claude/healing-hideaway-pos-progress.md`._

---

## 1. POC scope

| In scope (POC) | Out of scope (later) |
|---|---|
| Booking flow: Service → Time (week + full calendar) → Details → Done | Taking payment in the app (it happens on Lindsey's mobile device) |
| Bookings are **confirmed immediately** and start as **Unpaid** | Staff console screen for marking bookings paid |
| Slot availability from Lindsey's schedule, with **Unavailable** slots (no names) | Shared backend API + Microsoft SQL Server |
| Backend command for staff to mark a booking **Paid** (no UI yet) | Companion website |
| On-screen keyboard, Small/Medium/Large text, idle reset | Full console (calendar, clients, sales, refunds) |
| 1080p full-screen kiosk under `cage`, vertical-only scrolling | Email/SMS receipts and reminders, multi-device sync |

> **Note:** The POC keeps all data **in memory** behind small Rust modules, so you can later swap in the real API without changing the UI. It needs **no network**.

> **Payment rule (applies to the POC too):** The app never takes payment and never collects, displays or stores card numbers. Lindsey takes payment on her mobile device, and staff then mark the booking paid by hand.

> **Open question: gift cards.** It isn't decided yet how gift cards are sold, issued or redeemed, so they're left out of the POC. See [section 11](#11-open-questions).

---

## 2. Stack

| Layer | Choice | Why |
|---|---|---|
| App shell | **Tauri 2** (Rust) | Small binaries, Rust backend, Linux `.rpm` bundling |
| UI | **Svelte 5 + SvelteKit (static) + TypeScript** | Shared later with the website; uses your TypeScript skills |
| Dates/times | `chrono` | Slot math, month views |
| IDs | `uuid` (v4) | Bookings |
| Fonts | `@fontsource/dm-sans`, `@fontsource/dm-serif-display` | Bundled locally; the kiosk needs no internet |
| Kiosk session | `cage` (Wayland kiosk compositor) | One full-screen app, nothing else reachable |

---

## 3. Dev environment (Fedora)

### 3.1 System packages (Tauri 2 prerequisites)

```bash
sudo dnf check-update
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel libxdo-devel
sudo dnf group install "c-development"
```

> **Note:** Check these against the current Tauri "Prerequisites → Linux → Fedora" docs before installing, since package names can change between releases.

### 3.2 Rust and Node

```bash
# Rust (skip if already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Node LTS (or use your preferred version manager)
sudo dnf install nodejs npm
node -v && npm -v
```

### 3.3 Kiosk compositor (for testing the real kiosk session)

```bash
sudo dnf install cage
```

---

## 4. Scaffold the project

```bash
npm create tauri-app@latest
# Project name:        healing-hideaway-kiosk
# Identifier:          com.healinghideaway.kiosk
# Frontend language:   TypeScript / JavaScript
# Package manager:     npm
# UI template:         Svelte
# UI flavor:           TypeScript

cd healing-hideaway-kiosk
npm install
npm install @fontsource/dm-sans @fontsource/dm-serif-display
npm run tauri dev   # sanity check: the template window opens
```

### 4.1 Target structure

```
healing-hideaway-kiosk/
├── src/                          # Svelte UI
│   ├── app.css                   # design tokens + kiosk hardening
│   ├── lib/
│   │   ├── api.ts                # typed invoke() wrappers
│   │   ├── flow.svelte.ts        # flow state (screen, selections, text size)
│   │   ├── idle.ts               # idle reset timer
│   │   ├── components/
│   │   │   ├── TopBar.svelte
│   │   │   ├── Keyboard.svelte
│   │   │   └── TextSizePicker.svelte
│   │   └── screens/
│   │       ├── Home.svelte  ServiceStep.svelte  TimeStep.svelte
│   │       ├── CalendarStep.svelte  DetailsStep.svelte
│   │       └── DoneStep.svelte
│   └── routes/
│       ├── +layout.ts            # (template) prerender = true, ssr = false
│       ├── +layout.svelte
│       └── +page.svelte          # screen router
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    └── src/
        ├── main.rs               # (template) calls lib::run()
        ├── lib.rs
        ├── domain.rs
        ├── availability.rs
        ├── store.rs
        └── commands.rs
```

---

## 5. Rust backend

### 5.1 `src-tauri/Cargo.toml` (dependencies)

Keep the template's `[package]`, `[lib]` and `[build-dependencies]` sections, and set the package `name` to `healing-hideaway-kiosk`. Then add:

```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
# keep any plugin lines the template added (e.g. tauri-plugin-opener)
```

### 5.2 `domain.rs`: shared types

```rust
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceOption {
    pub minutes: u32,
    pub price_cents: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub options: Vec<ServiceOption>,
}

/// Clients only ever see these two states, never who booked a slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SlotStatus {
    Open,
    Unavailable,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub start: String, // "16:00" (24h, sent back when booking)
    pub label: String, // "4:00 PM"
    pub status: SlotStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DayStatus {
    Past,
    Closed,
    Open,
    Limited, // 1–2 open slots ("2 left")
    Full,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayAvailability {
    pub date: NaiveDate,
    pub status: DayStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDetails {
    pub name: String,
    pub phone: String,
    pub email: String,
}

/// Payment is taken outside the app (on Lindsey's mobile device).
/// Staff flip Unpaid → Paid by hand; nothing in the app charges a card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Unpaid,
    Paid,
}

#[derive(Clone, Debug)]
pub struct Booking {
    pub id: Uuid,
    pub service_id: u32,
    pub minutes: u32,
    pub start: NaiveDateTime,
    pub price_cents: i64,      // snapshot at booking time
    pub client: ClientDetails, // stays on the device/server, never sent to slot queries
    pub payment: PaymentStatus,
    pub paid_at: Option<NaiveDateTime>,
}

/// What the Done screen shows. No client details.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingConfirmation {
    pub id: Uuid,
    pub service_name: String,
    pub minutes: u32,
    pub date: NaiveDate,
    pub start_label: String, // "4:00 PM"
    pub price_cents: i64,
}
```

### 5.3 `availability.rs`: schedule and slot math

```rust
use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Weekday};

use crate::domain::{DayAvailability, DayStatus, Slot, SlotStatus};

#[derive(Clone)]
pub struct DayHours {
    pub open: NaiveTime,
    pub close: NaiveTime,
    pub lunch: Option<(NaiveTime, NaiveTime)>,
}

#[derive(Clone)]
pub struct Schedule {
    pub weekly: HashMap<Weekday, DayHours>,         // missing day = closed
    pub buffer: Duration,                           // room reset between clients
    pub step: Duration,                             // spacing between start times
    pub min_notice: Duration,                       // how far ahead a slot must be booked
    pub blocks: Vec<(NaiveDateTime, NaiveDateTime)>, // time off / personal blocks
}

impl Schedule {
    /// Mon, Wed, Sat, Sun. Hours/breaks are EXAMPLES — replace with Lindsey's.
    pub fn healing_hideaway_default() -> Self {
        let t = |h, m| NaiveTime::from_hms_opt(h, m, 0).unwrap();
        let lunch = Some((t(12, 0), t(12, 30)));
        let mut weekly = HashMap::new();
        weekly.insert(Weekday::Mon, DayHours { open: t(9, 0), close: t(19, 0), lunch });
        weekly.insert(Weekday::Wed, DayHours { open: t(9, 0), close: t(19, 0), lunch });
        weekly.insert(Weekday::Sat, DayHours { open: t(9, 0), close: t(17, 0), lunch });
        weekly.insert(Weekday::Sun, DayHours { open: t(11, 0), close: t(17, 0), lunch: None });
        Self {
            weekly,
            buffer: Duration::minutes(15),
            step: Duration::minutes(30), // fits 30, 60 and 90 minute services
            min_notice: Duration::hours(24), // no same-day bookings
            blocks: Vec::new(),
        }
    }
}

fn overlaps(a: (NaiveDateTime, NaiveDateTime), b: (NaiveDateTime, NaiveDateTime)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

/// `busy` = existing bookings (start, end) that still hold their slot.
pub fn slots_for_day(
    date: NaiveDate,
    minutes: u32,
    schedule: &Schedule,
    busy: &[(NaiveDateTime, NaiveDateTime)],
    now: NaiveDateTime,
) -> Vec<Slot> {
    let Some(hours) = schedule.weekly.get(&date.weekday()) else {
        return Vec::new(); // closed day
    };
    let len = Duration::minutes(minutes as i64);
    let close = date.and_time(hours.close);
    let mut start = date.and_time(hours.open);
    let mut out = Vec::new();

    while start + len <= close {
        let end = start + len;
        let lunch = hours
            .lunch
            .map(|(a, b)| overlaps((start, end), (date.and_time(a), date.and_time(b))))
            .unwrap_or(false);
        let booked = busy
            .iter()
            .any(|&(bs, be)| overlaps((start, end + schedule.buffer), (bs, be + schedule.buffer)));
        let blocked = schedule.blocks.iter().any(|&blk| overlaps((start, end), blk));
        let too_soon = start < now + schedule.min_notice;

        let status = if lunch || booked || blocked || too_soon {
            SlotStatus::Unavailable
        } else {
            SlotStatus::Open
        };
        out.push(Slot {
            start: start.format("%H:%M").to_string(),
            label: start.format("%-I:%M %p").to_string(),
            status,
        });
        start += schedule.step;
    }
    out
}

pub fn month_availability(
    year: i32,
    month: u32,
    minutes: u32,
    schedule: &Schedule,
    busy: &[(NaiveDateTime, NaiveDateTime)],
    now: NaiveDateTime,
) -> Vec<DayAvailability> {
    let Some(mut d) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    while d.month() == month {
        let status = if d < now.date() {
            DayStatus::Past
        } else if !schedule.weekly.contains_key(&d.weekday()) {
            DayStatus::Closed
        } else {
            let open = slots_for_day(d, minutes, schedule, busy, now)
                .iter()
                .filter(|s| s.status == SlotStatus::Open)
                .count();
            match open {
                0 => DayStatus::Full,
                1..=2 => DayStatus::Limited,
                _ => DayStatus::Open,
            }
        };
        out.push(DayAvailability { date: d, status });
        d = d.succ_opt().expect("date overflow");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d).unwrap().and_hms_opt(h, min, 0).unwrap()
    }

    #[test]
    fn tuesday_is_closed() {
        let s = Schedule::healing_hideaway_default();
        let tue = NaiveDate::from_ymd_opt(2026, 11, 24).unwrap();
        assert!(slots_for_day(tue, 60, &s, &[], dt(2026, 11, 1, 8, 0)).is_empty());
    }

    #[test]
    fn booked_slot_is_unavailable_without_details() {
        let s = Schedule::healing_hideaway_default();
        let sat = NaiveDate::from_ymd_opt(2026, 11, 21).unwrap();
        let busy = [(dt(2026, 11, 21, 10, 30), dt(2026, 11, 21, 11, 30))];
        let slots = slots_for_day(sat, 60, &s, &busy, dt(2026, 11, 1, 8, 0));
        let s1030 = slots.iter().find(|x| x.start == "10:30").unwrap();
        assert_eq!(s1030.status, SlotStatus::Unavailable);
    }

    #[test]
    fn lunch_blocks_noon() {
        let s = Schedule::healing_hideaway_default();
        let mon = NaiveDate::from_ymd_opt(2026, 11, 23).unwrap();
        let slots = slots_for_day(mon, 60, &s, &[], dt(2026, 11, 1, 8, 0));
        assert_eq!(slots.iter().find(|x| x.start == "12:00").unwrap().status, SlotStatus::Unavailable);
    }

    #[test]
    fn buffer_pushes_next_start_to_the_half_hour() {
        let s = Schedule::healing_hideaway_default();
        let mon = NaiveDate::from_ymd_opt(2026, 11, 23).unwrap();
        let busy = [(dt(2026, 11, 23, 9, 0), dt(2026, 11, 23, 10, 0))]; // 60 min at 9:00
        let slots = slots_for_day(mon, 60, &s, &busy, dt(2026, 11, 1, 8, 0));
        let status = |t: &str| slots.iter().find(|x| x.start == t).unwrap().status;
        assert_eq!(status("10:00"), SlotStatus::Unavailable); // inside the 15 min reset
        assert_eq!(status("10:30"), SlotStatus::Open);
    }
}
```

### 5.4 `store.rs`: in-memory data

```rust
use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{Duration, NaiveDateTime};
use uuid::Uuid;

use crate::availability::Schedule;
use crate::domain::*;

pub struct Store {
    pub services: Vec<Service>,
    pub schedule: Schedule,
    pub bookings: HashMap<Uuid, Booking>,
}

pub struct AppState(pub Mutex<Store>);

impl Store {
    pub fn seed() -> Self {
        // PLACEHOLDER prices and lengths: replace with Lindsey's real menu.
        // Lengths must be 30, 60 or 90 minutes.
        let svc = |id, name: &str, desc: &str, opts: &[(u32, i64)]| Service {
            id,
            name: name.into(),
            description: desc.into(),
            options: opts.iter().map(|&(minutes, price_cents)| ServiceOption { minutes, price_cents }).collect(),
        };
        Self {
            services: vec![
                svc(1, "Swedish", "Long, flowing strokes to relax the whole body", &[(30, 0), (60, 0), (90, 0)]),
                svc(2, "Deep tissue", "Firm, focused pressure for tension and knots", &[(60, 0), (90, 0)]),
                svc(3, "Hot stone", "Heated stones to warm and loosen muscles", &[(90, 0)]),
                svc(4, "Prenatal", "Side-lying and supported for comfort", &[(60, 0)]),
            ],
            schedule: Schedule::healing_hideaway_default(),
            bookings: HashMap::new(),
        }
    }

    /// Every booking holds its slot, paid or not (bookings are confirmed immediately).
    pub fn busy(&self) -> Vec<(NaiveDateTime, NaiveDateTime)> {
        self.bookings
            .values()
            .map(|b| (b.start, b.start + Duration::minutes(b.minutes as i64)))
            .collect()
    }

    /// Staff action after taking payment on the mobile device.
    /// Idempotent: marking a paid booking again keeps the first `paid_at`.
    pub fn mark_paid(&mut self, id: Uuid, at: NaiveDateTime) -> Result<(), String> {
        let b = self.bookings.get_mut(&id).ok_or("Unknown booking")?;
        if b.payment == PaymentStatus::Unpaid {
            b.payment = PaymentStatus::Paid;
            b.paid_at = Some(at);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn dt(h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 23).unwrap().and_hms_opt(h, m, 0).unwrap()
    }

    fn store_with_booking() -> (Store, Uuid) {
        let mut s = Store::seed();
        let id = Uuid::new_v4();
        s.bookings.insert(id, Booking {
            id,
            service_id: 1,
            minutes: 60,
            start: dt(9, 0),
            price_cents: 0,
            client: ClientDetails { name: "Test".into(), phone: String::new(), email: String::new() },
            payment: PaymentStatus::Unpaid,
            paid_at: None,
        });
        (s, id)
    }

    #[test]
    fn unpaid_booking_still_holds_its_slot() {
        let (s, _) = store_with_booking();
        assert_eq!(s.busy(), vec![(dt(9, 0), dt(10, 0))]);
    }

    #[test]
    fn mark_paid_is_idempotent() {
        let (mut s, id) = store_with_booking();
        s.mark_paid(id, dt(10, 0)).unwrap();
        s.mark_paid(id, dt(11, 0)).unwrap();
        assert_eq!(s.bookings[&id].payment, PaymentStatus::Paid);
        assert_eq!(s.bookings[&id].paid_at, Some(dt(10, 0)));
    }

    #[test]
    fn mark_paid_rejects_unknown_booking() {
        let (mut s, _) = store_with_booking();
        assert!(s.mark_paid(Uuid::new_v4(), dt(10, 0)).is_err());
    }
}
```

> **Note:** Set the placeholder `price_cents` values to non-zero amounts for testing; zero is used here so no invented prices ship by accident.

### 5.5 `commands.rs`: what the UI can call

```rust
use chrono::{Local, NaiveDate, NaiveTime};
use tauri::State;
use uuid::Uuid;

use crate::availability::{month_availability, slots_for_day};
use crate::domain::*;
use crate::store::AppState;

#[tauri::command]
pub fn list_services(state: State<'_, AppState>) -> Vec<Service> {
    state.0.lock().unwrap().services.clone()
}

#[tauri::command]
pub fn day_slots(state: State<'_, AppState>, date: NaiveDate, minutes: u32) -> Vec<Slot> {
    let store = state.0.lock().unwrap();
    slots_for_day(date, minutes, &store.schedule, &store.busy(), Local::now().naive_local())
}

#[tauri::command]
pub fn month_days(state: State<'_, AppState>, year: i32, month: u32, minutes: u32) -> Vec<DayAvailability> {
    let store = state.0.lock().unwrap();
    month_availability(year, month, minutes, &store.schedule, &store.busy(), Local::now().naive_local())
}

/// Confirms the booking immediately. It starts Unpaid; payment happens on
/// Lindsey's mobile device and staff mark it paid afterwards.
#[tauri::command]
pub fn create_booking(
    state: State<'_, AppState>,
    service_id: u32,
    minutes: u32,
    date: NaiveDate,
    start: String,
    client: ClientDetails,
) -> Result<BookingConfirmation, String> {
    let mut store = state.0.lock().unwrap();
    let (service_name, price_cents) = store
        .services
        .iter()
        .find(|s| s.id == service_id)
        .and_then(|s| {
            s.options
                .iter()
                .find(|o| o.minutes == minutes)
                .map(|o| (s.name.clone(), o.price_cents))
        })
        .ok_or("Unknown service or length")?;

    // Re-check on the backend: never trust the screen's view of availability.
    let now = Local::now().naive_local();
    let slot = slots_for_day(date, minutes, &store.schedule, &store.busy(), now)
        .into_iter()
        .find(|s| s.start == start && s.status == SlotStatus::Open)
        .ok_or("That time was just taken. Please pick another.")?;

    let t = NaiveTime::parse_from_str(&start, "%H:%M").map_err(|e| e.to_string())?;
    let id = Uuid::new_v4();
    store.bookings.insert(
        id,
        Booking {
            id,
            service_id,
            minutes,
            start: date.and_time(t),
            price_cents,
            client,
            payment: PaymentStatus::Unpaid,
            paid_at: None,
        },
    );
    Ok(BookingConfirmation { id, service_name, minutes, date, start_label: slot.label, price_cents })
}

/// Staff only: call after taking payment on the mobile device.
/// No kiosk screen calls this. It moves behind staff sign-in with the console.
#[tauri::command]
pub fn mark_booking_paid(state: State<'_, AppState>, booking_id: Uuid) -> Result<(), String> {
    state.0.lock().unwrap().mark_paid(booking_id, Local::now().naive_local())
}
```

> **Warning:** In the POC, `mark_booking_paid` has no sign-in in front of it, so anything running in the webview can call it. That's fine for testing. Before the app takes real bookings, it must only be reachable after staff sign-in.

### 5.6 `lib.rs`: wire it together

```rust
mod availability;
mod commands;
mod domain;
mod store;

use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // keep plugin lines from the template here, e.g. .plugin(tauri_plugin_opener::init())
        .manage(store::AppState(Mutex::new(store::Store::seed())))
        .invoke_handler(tauri::generate_handler![
            commands::list_services,
            commands::day_slots,
            commands::month_days,
            commands::create_booking,
            commands::mark_booking_paid,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Healing Hideaway kiosk");
}
```

```bash
cd src-tauri && cargo test   # runs the availability and store tests
```

### 5.7 `tauri.conf.json`: 1080p kiosk window

```json
{
  "productName": "Healing Hideaway",
  "identifier": "com.healinghideaway.kiosk",
  "app": {
    "windows": [
      {
        "title": "Healing Hideaway",
        "width": 1920,
        "height": 1080,
        "resizable": false,
        "decorations": false,
        "fullscreen": true,
        "zoomHotkeysEnabled": false
      }
    ]
  }
}
```

> **Tip:** Set `"fullscreen": false` while developing on your workstation, and set it back to `true` for the kiosk build. Merge these keys into the template's file rather than replacing it, so its `build` and `bundle` sections stay intact.

---

## 6. Svelte UI

### 6.1 `src/app.css`: tokens, text sizes, kiosk hardening

```css
:root {
  --bg: #16151b;
  --surface: #221f29;
  --surface-2: #2a2533;
  --border: #3a3542;
  --border-strong: #4a4453;
  --text: #f3eee6;
  --muted: #b8b0a4;
  --dim: #8a8295;
  --accent: #c9b6e8;
  --on-accent: #16151b;
  --font-display: 'DM Serif Display', serif;
  --font-body: 'DM Sans', system-ui, sans-serif;
  --ui-scale: 1; /* set from JS: window height / 800 (1080p → 1.35) */
}

/* Text size: everything uses rem, so one value scales the whole UI */
html                         { font-size: calc(18px * var(--ui-scale)); }
html[data-text-size="small"] { font-size: calc(16px * var(--ui-scale)); }
html[data-text-size="large"] { font-size: calc(22px * var(--ui-scale)); }

*, *::before, *::after { box-sizing: border-box; }

html, body {
  margin: 0;
  height: 100%;
  overflow: hidden;              /* the app scrolls inside <main>, never the window */
  background: var(--bg);
  color: var(--text);
  font-family: var(--font-body);
}

.kiosk {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  user-select: none;
  -webkit-user-select: none;
  touch-action: manipulation;    /* no double-tap zoom */
}

/* Vertical scrolling only */
.kiosk main {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  touch-action: pan-y;
  padding: 1.5rem 2.5rem;
}

h1, h2, .display { font-family: var(--font-display); font-weight: 400; margin: 0; }

button {
  font: inherit;
  min-height: 3.5rem;            /* ≥ 60px at Medium on 1080p */
  border-radius: 0.9rem;
  cursor: pointer;
  color: var(--text);
  background: var(--surface);
  border: 1px solid var(--border);
}
button:hover:not(:disabled) { filter: brightness(1.1); box-shadow: 0 0 0 2px var(--accent); }
button:active:not(:disabled) { transform: translateY(1px); }
button:focus-visible, input:focus-visible, a:focus-visible { outline: 3px solid var(--text); outline-offset: 3px; }
button.primary { background: var(--accent); color: var(--on-accent); border: none; font-weight: 700; }

input {
  font: inherit;
  user-select: text;
  -webkit-user-select: text;
  height: 3.2rem;
  padding: 0 1rem;
  border-radius: 0.8rem;
  border: 1px solid var(--border-strong);
  background: var(--surface);
  color: var(--text);
}
input.active { border: 2px solid var(--accent); box-shadow: 0 0 0 4px #3a3150; }

/* Grids drop columns at Large text instead of overflowing sideways */
.slot-grid { display: grid; gap: 0.6rem; grid-template-columns: repeat(4, minmax(0, 1fr)); }
html[data-text-size="large"] .slot-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }

.slot.unavailable {
  background: transparent;
  border: 1px dashed var(--border);
  color: var(--dim);
  cursor: not-allowed;
}
.slot.unavailable .time { text-decoration: line-through; }
```

### 6.2 `src/routes/+layout.svelte`

```svelte
<script lang="ts">
  import '@fontsource/dm-sans/400.css';
  import '@fontsource/dm-sans/700.css';
  import '@fontsource/dm-serif-display/400.css';
  import '../app.css';
  import { onMount } from 'svelte';

  let { children } = $props();

  onMount(() => {
    // 1080p design: the UI is laid out at 1422×800 logical px and scaled to fit
    const setScale = () =>
      document.documentElement.style.setProperty('--ui-scale', String(window.innerHeight / 800));
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
```

### 6.3 `src/lib/api.ts`: typed commands

```ts
import { invoke } from '@tauri-apps/api/core';

export type SlotStatus = 'open' | 'unavailable';
export type DayStatus = 'past' | 'closed' | 'open' | 'limited' | 'full';
export type Minutes = 30 | 60 | 90;

export interface ServiceOption { minutes: Minutes; priceCents: number }
export interface Service { id: number; name: string; description: string; options: ServiceOption[] }
export interface Slot { start: string; label: string; status: SlotStatus }
export interface DayAvailability { date: string; status: DayStatus }
export interface ClientDetails { name: string; phone: string; email: string }
export interface BookingConfirmation {
  id: string; serviceName: string; minutes: Minutes; date: string; startLabel: string; priceCents: number;
}

// Tauri maps camelCase argument keys to the Rust snake_case parameter names.
export const api = {
  services: () => invoke<Service[]>('list_services'),
  daySlots: (date: string, minutes: number) => invoke<Slot[]>('day_slots', { date, minutes }),
  monthDays: (year: number, month: number, minutes: number) =>
    invoke<DayAvailability[]>('month_days', { year, month, minutes }),
  createBooking: (serviceId: number, minutes: number, date: string, start: string, client: ClientDetails) =>
    invoke<BookingConfirmation>('create_booking', { serviceId, minutes, date, start, client }),
  // Staff only. Not called by any kiosk screen; used later by the staff console.
  markBookingPaid: (bookingId: string) => invoke<void>('mark_booking_paid', { bookingId }),
};
```

### 6.4 `src/lib/flow.svelte.ts`: flow state

```ts
import type { BookingConfirmation, Minutes } from './api';

export type Screen =
  | 'home' | 'service' | 'time' | 'calendar' | 'details' | 'done';
export type TextSize = 'small' | 'medium' | 'large';
export type Field = 'name' | 'phone' | 'email';

function initial() {
  return {
    screen: 'home' as Screen,
    textSize: 'medium' as TextSize,
    serviceId: null as number | null,
    minutes: 60 as Minutes,
    date: '',          // 'YYYY-MM-DD'
    start: '',         // 'HH:MM'
    startLabel: '',    // '4:00 PM'
    client: { name: '', phone: '', email: '' },
    activeField: null as Field | null,
    confirmation: null as BookingConfirmation | null, // set when the booking is created
  };
}

export const flow = $state(initial());

/** Clears everything, including text size, for the next client. */
export function resetFlow() {
  Object.assign(flow, initial());
}

$effect.root(() => {
  $effect(() => {
    document.documentElement.dataset.textSize = flow.textSize;
  });
});
```

### 6.5 `src/lib/idle.ts`: return to Home after inactivity

```ts
import { flow, resetFlow } from './flow.svelte';

export function startIdleTimer(ms = 90_000) {
  let timer: ReturnType<typeof setTimeout>;
  const events = ['pointerdown', 'keydown'] as const;

  const reset = () => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (flow.screen !== 'home') resetFlow();
    }, ms);
  };

  events.forEach((e) => window.addEventListener(e, reset, { passive: true }));
  reset();
  return () => {
    clearTimeout(timer);
    events.forEach((e) => window.removeEventListener(e, reset));
  };
}
```

### 6.6 `src/routes/+page.svelte`: screen router

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { flow, resetFlow, type Screen } from '$lib/flow.svelte';
  import { startIdleTimer } from '$lib/idle';
  import Home from '$lib/screens/Home.svelte';
  import ServiceStep from '$lib/screens/ServiceStep.svelte';
  import TimeStep from '$lib/screens/TimeStep.svelte';
  import CalendarStep from '$lib/screens/CalendarStep.svelte';
  import DetailsStep from '$lib/screens/DetailsStep.svelte';
  import DoneStep from '$lib/screens/DoneStep.svelte';

  onMount(() => startIdleTimer(90_000));

  // Esc = "go back one level": hide the on-screen keyboard first, then step back.
  // Matches each screen's Back button.
  const back: Partial<Record<Screen, Screen>> = {
    service: 'home',
    time: 'service',
    calendar: 'time',
    details: 'time',
  };

  function onKeydown(e: KeyboardEvent) {
    if (e.key !== 'Escape') return;
    e.preventDefault();
    if (flow.activeField) {
      flow.activeField = null;
      (document.activeElement as HTMLElement | null)?.blur();
    } else if (flow.screen === 'done') {
      resetFlow(); // same as the Done screen's own timeout
    } else {
      const to = back[flow.screen];
      if (to) flow.screen = to;
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="kiosk">
  {#if flow.screen === 'home'}<Home />
  {:else if flow.screen === 'service'}<ServiceStep />
  {:else if flow.screen === 'time'}<TimeStep />
  {:else if flow.screen === 'calendar'}<CalendarStep />
  {:else if flow.screen === 'details'}<DetailsStep />
  {:else if flow.screen === 'done'}<DoneStep />
  {/if}
</div>
```

### 6.7 `src/lib/screens/TimeStep.svelte`: slots with Unavailable state

```svelte
<script lang="ts">
  import { api, type Slot } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import TopBar from '$lib/components/TopBar.svelte';

  // Next 7 days starting tomorrow (24 h notice means today is never bookable)
  const days = Array.from({ length: 7 }, (_, i) => {
    const d = new Date();
    d.setDate(d.getDate() + i + 1);
    return {
      iso: d.toLocaleDateString('en-CA'), // YYYY-MM-DD in local time
      dow: d.toLocaleDateString('en-US', { weekday: 'short' }),
      num: d.getDate(),
    };
  });

  let slots = $state<Slot[]>([]);

  $effect(() => {
    if (!flow.date) flow.date = days[0].iso;
    api.daySlots(flow.date, flow.minutes).then((s) => (slots = s));
  });

  function pick(s: Slot) {
    if (s.status !== 'open') return;
    flow.start = s.start;
    flow.startLabel = s.label;
  }
</script>

<TopBar step={2} label="Time" />
<main>
  <div style="display:flex; justify-content:space-between; align-items:center">
    <h1 style="font-size:2.4rem">When works for you?</h1>
    <button onclick={() => (flow.screen = 'calendar')}>See full calendar ›</button>
  </div>

  <div style="display:grid; grid-template-columns:repeat(7, minmax(0,1fr)); gap:0.6rem; margin:1.2rem 0">
    {#each days as d}
      <button class:primary={flow.date === d.iso} onclick={() => { flow.date = d.iso; flow.start = ''; }}>
        {d.dow}<br /><span class="display" style="font-size:1.7rem">{d.num}</span>
      </button>
    {/each}
  </div>

  {#if slots.length === 0}
    <p style="color:var(--muted)">Closed this day. Please pick another date.</p>
  {:else}
    <div class="slot-grid">
      {#each slots as s}
        {#if s.status === 'unavailable'}
          <button class="slot unavailable" disabled aria-label="{s.label}, unavailable">
            <span class="time">{s.label}</span><br /><small>Unavailable</small>
          </button>
        {:else}
          <button class="slot" class:primary={flow.start === s.start} onclick={() => pick(s)}>{s.label}</button>
        {/if}
      {/each}
    </div>
  {/if}
</main>
<footer style="display:flex; gap:1rem; justify-content:flex-end; padding:1rem 2.5rem 1.5rem; border-top:1px solid var(--border)">
  <button onclick={() => (flow.screen = 'service')}>Back</button>
  <button class="primary" disabled={!flow.start} onclick={() => (flow.screen = 'details')}>Continue to your details</button>
</footer>
```

### 6.8 `src/lib/components/Keyboard.svelte`: on-screen keyboard

```svelte
<script lang="ts">
  import { flow, type Field } from '$lib/flow.svelte';

  let shift = $state(false);
  const rows = ['1234567890', 'qwertyuiop', 'asdfghjkl', 'zxcvbnm'].map((r) => r.split(''));
  // Needed for emails (first_last@, name+tag@, hyphenated domains) and names (O'Brien, Mary-Kate)
  const symbols = ['-', '_', '+', "'"];
  // Phone gets a number pad instead of letters
  const pad = [['1', '2', '3'], ['4', '5', '6'], ['7', '8', '9'], ['+', '0', '-']];
  const order: Field[] = ['name', 'phone', 'email'];

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
  // Keep focus in the input while tapping keys
  const keep = (e: PointerEvent) => e.preventDefault();
</script>

{#if flow.activeField}
  <div class="kb" role="group" aria-label="On-screen keyboard">
    <div class="kb-head">
      <span>Typing in: {flow.activeField}</span>
      <span>
        <button onpointerdown={keep} onclick={() => move(-1)}>‹ Previous</button>
        <button onpointerdown={keep} onclick={() => move(1)}>Next field ›</button>
        <button onpointerdown={keep} onclick={() => (flow.activeField = null)} aria-label="Hide keyboard">Hide ⌄</button>
      </span>
    </div>
    {#if flow.activeField === 'phone'}
      <div class="pad">
        {#each pad as row}
          <div class="kb-row">
            {#each row as ch}
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
      {#each rows as row, i}
        <div class="kb-row">
          {#if i === 3}<button class="mod" onpointerdown={keep} aria-pressed={shift} onclick={() => (shift = !shift)} aria-label="Shift">⇧</button>{/if}
          {#each row as ch}
            <button onpointerdown={keep} onclick={() => type(ch)}>{shift ? ch.toUpperCase() : ch}</button>
          {/each}
          {#if i === 3}<button class="mod" onpointerdown={keep} onclick={backspace} aria-label="Backspace">⌫</button>{/if}
        </div>
      {/each}
      <div class="kb-row">
        <button onpointerdown={keep} onclick={() => type('@')}>@</button>
        {#each symbols as ch}
          <button onpointerdown={keep} onclick={() => type(ch)}>{ch}</button>
        {/each}
        <button style="flex:4" onpointerdown={keep} onclick={() => type(' ')} aria-label="Space">space</button>
        <button onpointerdown={keep} onclick={() => type('.')}>.</button>
        <button onpointerdown={keep} onclick={() => type('.com')}>.com</button>
        <button class="primary" style="flex:1.8" onpointerdown={keep} onclick={() => move(1)}>Continue</button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .kb { background: #1d1b23; border-top: 1px solid var(--border); padding: 0.8rem 2.5rem 1.2rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .kb-head { display: flex; justify-content: space-between; align-items: center; font-weight: 700; }
  .kb-row { display: flex; gap: 0.4rem; }
  .kb-row button { flex: 1; min-height: 2.9rem; background: #2e2a36; border: none; }
  .kb-row .mod { flex: 1.5; background: var(--border); }
  .pad { width: min(100%, 28rem); margin: 0 auto; display: flex; flex-direction: column; gap: 0.5rem; }
  .pad .kb-row button { min-height: 3.4rem; font-size: 1.3rem; }
  .pad .kb-row .mod { flex: 1; }
</style>
```

> **Note:** `DetailsStep.svelte` (6.9) sets `flow.activeField` when an input gets focus. A physical keyboard keeps working normally. Add a setting later to hide the on-screen keyboard when one is attached.

### 6.9 `src/lib/screens/DetailsStep.svelte`: confirm the booking

There is no Pay step. Confirming here creates the booking immediately (as Unpaid) and goes straight to Done.

```svelte
<script lang="ts">
  import { api, type ClientDetails } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import Keyboard from '$lib/components/Keyboard.svelte';

  const fields: { key: keyof ClientDetails; label: string; type: string }[] = [
    { key: 'name', label: 'Name', type: 'text' },
    { key: 'phone', label: 'Phone', type: 'tel' },
    { key: 'email', label: 'Email', type: 'email' },
  ];

  let error = $state('');
  let saving = $state(false);
  // PLACEHOLDER rule: name plus a phone or email. Adjust to what Lindsey needs.
  const ready = $derived(
    flow.client.name.trim() !== '' && (flow.client.phone.trim() !== '' || flow.client.email.trim() !== ''),
  );

  async function confirm() {
    if (saving || !ready || flow.serviceId === null) return;
    saving = true;
    error = '';
    try {
      flow.confirmation = await api.createBooking(flow.serviceId, flow.minutes, flow.date, flow.start, flow.client);
      flow.activeField = null;
      flow.screen = 'done';
    } catch (e) {
      error = String(e); // e.g. "That time was just taken. Please pick another."
    } finally {
      saving = false;
    }
  }
</script>

<TopBar step={3} label="Your details" />
<main>
  <h1 style="font-size:2.4rem">Your details</h1>
  <p style="color:var(--muted)">{flow.startLabel} · {flow.minutes} min. Payment is taken at your appointment.</p>

  {#each fields as f}
    <label style="display:flex; flex-direction:column; gap:0.4rem; margin-top:1rem">
      {f.label}
      <input
        type={f.type}
        bind:value={flow.client[f.key]}
        class:active={flow.activeField === f.key}
        onfocus={() => (flow.activeField = f.key)}
      />
    </label>
  {/each}

  {#if error}
    <p role="alert" style="margin-top:1rem">{error}</p>
    <button onclick={() => { flow.start = ''; flow.screen = 'time'; }}>Pick another time</button>
  {/if}
</main>
<Keyboard />
<footer style="display:flex; gap:1rem; justify-content:flex-end; padding:1rem 2.5rem 1.5rem; border-top:1px solid var(--border)">
  <button onclick={() => (flow.screen = 'time')}>Back</button>
  <button class="primary" disabled={!ready || saving} onclick={confirm}>Confirm booking</button>
</footer>
```

### 6.10 Remaining screens

Build these from the canvas mockups, using the same patterns:

| Screen | Key behavior |
|---|---|
| `Home.svelte` | Book tile (`flow.screen = 'service'`); hidden **Staff sign-in** link (goes nowhere in the POC). No gift card tiles until that's decided |
| `TopBar.svelte` | Brand, "Step N of 3 · Label", Start over (`resetFlow()`), **Aa Text size** button that opens `TextSizePicker` |
| `TextSizePicker.svelte` | Three `role="radio"` buttons that set `flow.textSize`; resets to Medium on `resetFlow()`. Esc closes it: handle `keydown` on the picker and call `e.stopPropagation()` so the router's Esc (6.6) doesn't also go back a screen |
| `ServiceStep.svelte` | `api.services()` → cards; length chips (only the 30 / 60 / 90 options that service offers) set `flow.minutes` |
| `CalendarStep.svelte` | `api.monthDays(y, m, flow.minutes)` → 7-column month grid (Past / Closed / Open / "2 left" / Full); day tap sets `flow.date` and shows the slots panel |
| `DoneStep.svelte` | Shows `flow.confirmation` (service, date, time, length, price) and "Payment is taken at your appointment"; auto `resetFlow()` after 30 s |

---

## 7. Run it

```bash
npm run tauri dev          # dev window with hot reload
cd src-tauri && cargo test # availability and store tests
npm run tauri build        # release build + .rpm in src-tauri/target/release/bundle/
```

To try marking a booking paid before the staff console exists, open the dev tools in `tauri dev` and run `window.__TAURI__.core.invoke('mark_booking_paid', { bookingId: '<id>' })`. This needs `"withGlobalTauri": true` under `app` in `tauri.conf.json`, for dev only.

---

## 8. Kiosk session on the in-office machine

### 8.1 Install and try it manually

```bash
sudo dnf install ./src-tauri/target/release/bundle/rpm/*.rpm
# From a text console (Ctrl+Alt+F3), logged in as the kiosk user:
cage -s -- healing-hideaway-kiosk   # -s allows Ctrl+Alt+F<n> so you can get back out
```

> **Warning:** `-s` turns on VT switching, which lets anyone with a keyboard reach a text console. Use it only while testing by hand. The boot service in 8.2 leaves it off.

> **Tip:** If the window comes up blank on some GPUs, start it with `WEBKIT_DISABLE_DMABUF_RENDERER=1`. This is a known WebKitGTK workaround.

### 8.2 Start at boot (systemd)

```bash
sudo useradd -m kiosk
sudo systemctl disable gdm   # the kiosk replaces the desktop login
```

```ini
# /etc/systemd/system/kiosk.service
[Unit]
Description=Healing Hideaway kiosk
After=systemd-user-sessions.service plymouth-quit-wait.service
Conflicts=getty@tty1.service

[Service]
User=kiosk
PAMName=login
TTYPath=/dev/tty1
TTYReset=yes
TTYVHangup=yes
TTYVTDisallocate=yes
UtmpIdentifier=tty1
StandardInput=tty-fail
Environment=WEBKIT_DISABLE_DMABUF_RENDERER=1
ExecStart=/usr/bin/cage -- /usr/bin/healing-hideaway-kiosk
Restart=always
RestartSec=2

[Install]
WantedBy=graphical.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now kiosk.service
```

> **Note:** Compare this unit with the `cage` project's systemd example, and check the installed binary name with `rpm -ql`. `Restart=always` brings the app back if it ever exits.

---

## 9. Acceptance checklist

| # | Check | Pass |
|---|---|---|
| 1 | Runs full-screen at **1920×1080**, with no window chrome and no desktop reachable; Ctrl+Alt+F2–F6 does nothing in the boot session | ☐ |
| 2 | **No horizontal scrolling** on any screen at Small, Medium or Large | ☐ |
| 3 | Large text: grids drop to 2 columns; long content scrolls **vertically** inside `main` | ☐ |
| 4 | **Touch only:** finish a booking | ☐ |
| 5 | **Mouse only:** same, with hover feedback and no right-click menu | ☐ |
| 6 | **Keyboard only:** Tab / Enter reach everything, and the focus outline is always visible. Esc closes the text size picker, then hides the on-screen keyboard, then goes back one screen (Done → Home) | ☐ |
| 7 | On-screen keyboard types into the active field, including `- _ + '`; the phone field shows a number pad; Previous / Next / Hide work | ☐ |
| 8 | Closed days (Tue, Thu, Fri) show as **Closed**; past days are disabled | ☐ |
| 9 | Booked times and lunch show **Unavailable**, with no names or details anywhere in the UI or command responses | ☐ |
| 10 | Booking the same slot twice: the second attempt is rejected by the backend | ☐ |
| 11 | Confirm booking → Done screen right away, showing the booking and "Payment is taken at your appointment" | ☐ |
| 12 | A new booking is **Unpaid**; `mark_booking_paid` sets it to Paid, and calling it twice changes nothing (`cargo test`) | ☐ |
| 13 | Start times fall on a 30-minute grid; each service offers only its 30 / 60 / 90 lengths | ☐ |
| 14 | Idle 90 s on any screen → back to Home with everything cleared | ☐ |
| 15 | Text size resets to Medium for the next client | ☐ |
| 16 | No payment or card fields exist anywhere in the app | ☐ |

---

## 10. After the POC

1. Move `store.rs` behind a **shared Axum API** backed by **Microsoft SQL Server**, following the persistence plan in this repo's `README.md`: `tiberius` with `bb8-tiberius` pooling, since `sqlx` dropped SQL Server support in 0.7. The kiosk never connects to the database directly. The UI keeps calling the same `api.ts` functions, which switch from `invoke()` to HTTP.
   - Put the store behind an async trait first (like the repo's `PosStore`), so the SQL Server version is a new implementation, not a rewrite.
   - Create the booking with the availability re-check and the insert in **one transaction**. Use an exclusive lock, for example `WITH (UPDLOCK, HOLDLOCK)` on the overlapping rows, so two devices can't book the same slot at once.
   - Types: booking IDs as `UNIQUEIDENTIFIER` (or `BIGINT IDENTITY` to match the repo), money as `BIGINT` cents, times as `DATETIME2`, payment status as a `CHECK`-constrained column.
   - SQL Server **Express** is free, with a 10 GB database limit, which is plenty here. On Fedora, run it from the `mcr.microsoft.com/mssql/server` container, since Microsoft's packages target RHEL and Ubuntu.
2. Add **staff sign-in** (PIN or passkey) leading to Lindsey's console: Today, Schedule (weekly hours, time off), Calendar, Clients, Sales. Move `mark_booking_paid` behind it, with a **Mark paid** button on each booking.
3. Add booking cancellation and no-show handling, since bookings are confirmed without payment.
4. Encrypt client details and intake notes at rest, and add audit logging (including who marked each booking paid).

---

## 11. Open questions

| Question | Notes |
|---|---|
| **Gift cards:** how are they sold, issued and redeemed? | Not decided, so not in the POC. Options include selling them through the mobile payment app's own gift card feature, or staff issuing a code in the console after taking payment. The answer decides whether the kiosk needs a balance check, and whether a booking can be paid by gift card. |
