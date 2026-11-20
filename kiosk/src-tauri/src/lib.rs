mod commands;

use std::sync::Mutex;

use kiosk_core::store::Store;

/// The app's single in-memory store. Commands lock it briefly per call.
pub struct AppState(pub Mutex<Store>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(Store::seed())))
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
