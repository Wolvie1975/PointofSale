//! What the UI can call. Thin wrappers: read the clock, lock the store,
//! call `kiosk-core`, and turn errors into the on-screen message.
//! All booking and staff rules live in `kiosk-core`.
//!
//! Client-facing commands never return client details or notes.
//! Every `staff_*` command except sign-in needs the token from `staff_sign_in`.

use chrono::{Local, NaiveDate, NaiveDateTime};
use kiosk_core::domain::{
    BookingConfirmation, ClientDetails, ClientRecord, ClientSummary, DayAvailability, Note, Service, Slot,
};
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

// ---- Staff only ----

/// Checks the staff session; locks only the staff gate, briefly.
fn authorize(gate: &State<'_, StaffGate>, token: &str) -> Result<(), String> {
    gate.0.lock().unwrap().authorize(token, now()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn staff_pin_configured(gate: State<'_, StaffGate>) -> bool {
    gate.0.lock().unwrap().is_configured()
}

/// Checks the PIN and returns a session token for the other staff commands.
#[tauri::command]
pub fn staff_sign_in(gate: State<'_, StaffGate>, pin: String) -> Result<String, String> {
    gate.0.lock().unwrap().sign_in(&pin, now()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn staff_sign_out(gate: State<'_, StaffGate>, token: String) {
    gate.0.lock().unwrap().sign_out(&token);
}

/// Closes the app. Under the kiosk's systemd service this restarts it fresh
/// (and clears all in-memory bookings and notes).
#[tauri::command]
pub fn staff_close_app(app: AppHandle, gate: State<'_, StaffGate>, token: String) -> Result<(), String> {
    authorize(&gate, &token)?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn staff_clients(
    state: State<'_, AppState>,
    gate: State<'_, StaffGate>,
    token: String,
    query: String,
) -> Result<Vec<ClientSummary>, String> {
    authorize(&gate, &token)?;
    Ok(state.0.lock().unwrap().list_clients(&query, now()))
}

#[tauri::command]
pub fn staff_client(
    state: State<'_, AppState>,
    gate: State<'_, StaffGate>,
    token: String,
    client_id: Uuid,
) -> Result<ClientRecord, String> {
    authorize(&gate, &token)?;
    state.0.lock().unwrap().client_record(client_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn staff_add_note(
    state: State<'_, AppState>,
    gate: State<'_, StaffGate>,
    token: String,
    client_id: Uuid,
    text: String,
) -> Result<Note, String> {
    authorize(&gate, &token)?;
    state.0.lock().unwrap().add_note(client_id, &text, now()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn staff_edit_note(
    state: State<'_, AppState>,
    gate: State<'_, StaffGate>,
    token: String,
    note_id: Uuid,
    text: String,
) -> Result<Note, String> {
    authorize(&gate, &token)?;
    state.0.lock().unwrap().edit_note(note_id, &text, now()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn staff_delete_note(
    state: State<'_, AppState>,
    gate: State<'_, StaffGate>,
    token: String,
    note_id: Uuid,
) -> Result<(), String> {
    authorize(&gate, &token)?;
    state.0.lock().unwrap().delete_note(note_id).map_err(|e| e.to_string())
}

/// After Lindsey takes payment on her mobile device.
#[tauri::command]
pub fn staff_mark_paid(
    state: State<'_, AppState>,
    gate: State<'_, StaffGate>,
    token: String,
    booking_id: Uuid,
) -> Result<(), String> {
    authorize(&gate, &token)?;
    state.0.lock().unwrap().mark_paid(booking_id, now()).map_err(|e| e.to_string())
}
