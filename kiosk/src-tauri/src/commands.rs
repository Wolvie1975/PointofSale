//! What the UI can call. Thin wrappers: read the clock, lock the store,
//! call `kiosk-core`, and turn errors into the client-facing message.
//! All booking rules live in `kiosk-core`.

use chrono::{Local, NaiveDate, NaiveDateTime};
use kiosk_core::domain::{BookingConfirmation, ClientDetails, DayAvailability, Service, Slot};
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::{AppState, StaffGate};

fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

#[tauri::command]
pub fn list_services(state: State<'_, AppState>) -> Vec<Service> {
    state.0.lock().unwrap().services.clone()
}

#[tauri::command]
pub fn day_slots(state: State<'_, AppState>, date: NaiveDate, minutes: u32) -> Vec<Slot> {
    state.0.lock().unwrap().day_slots(date, minutes, now())
}

#[tauri::command]
pub fn month_days(state: State<'_, AppState>, year: i32, month: u32, minutes: u32) -> Vec<DayAvailability> {
    state.0.lock().unwrap().month_days(year, month, minutes, now())
}

/// Confirms the booking immediately, as Unpaid.
#[tauri::command]
pub fn create_booking(
    state: State<'_, AppState>,
    service_id: u32,
    minutes: u32,
    date: NaiveDate,
    start: String,
    client: ClientDetails,
) -> Result<BookingConfirmation, String> {
    state
        .0
        .lock()
        .unwrap()
        .create_booking(service_id, minutes, date, &start, client, now())
        .map_err(|e| e.to_string())
}

/// Staff only: call after taking payment on the mobile device.
/// No kiosk screen calls this. WARNING: no sign-in in front of it yet; it must
/// move behind staff sign-in before the app takes real bookings.
#[tauri::command]
pub fn mark_booking_paid(state: State<'_, AppState>, booking_id: Uuid) -> Result<(), String> {
    state.0.lock().unwrap().mark_paid(booking_id, now()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn staff_pin_configured(gate: State<'_, StaffGate>) -> bool {
    gate.0.lock().unwrap().is_configured()
}

/// Staff only: closes the app after checking the PIN. Under the kiosk's systemd
/// service this restarts it fresh (and clears all in-memory bookings).
#[tauri::command]
pub fn staff_close_app(app: AppHandle, gate: State<'_, StaffGate>, pin: String) -> Result<(), String> {
    gate.0.lock().unwrap().check(&pin, now()).map_err(|e| e.to_string())?;
    app.exit(0);
    Ok(())
}
