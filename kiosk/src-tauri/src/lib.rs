mod commands;

use std::sync::Mutex;

use kiosk_core::staff::PinGate;
use kiosk_core::store::Store;

/// The app's single in-memory store. Commands lock it briefly per call.
pub struct AppState(pub Mutex<Store>);

/// Staff PIN gate (for closing the app).
pub struct StaffGate(pub Mutex<PinGate>);

/// The staff PIN comes from the `HH_STAFF_PIN` environment variable (4–8 digits),
/// so it's never in the code or the repo. Dev builds fall back to "0000" for
/// testing; release builds with no PIN set can't be closed from the screen.
fn staff_pin() -> Option<String> {
    std::env::var("HH_STAFF_PIN").ok().or_else(|| cfg!(debug_assertions).then(|| "0000".to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(Store::seed())))
        .manage(StaffGate(Mutex::new(PinGate::new(staff_pin()))))
        .invoke_handler(tauri::generate_handler![
            commands::list_services,
            commands::day_slots,
            commands::month_days,
            commands::create_booking,
            commands::mark_booking_paid,
            commands::staff_pin_configured,
            commands::staff_close_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Healing Hideaway kiosk");
}
