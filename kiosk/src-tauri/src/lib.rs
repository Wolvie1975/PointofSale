mod commands;

use std::sync::Mutex;

use kiosk_core::staff::StaffAuth;
use kiosk_core::store::Store;

/// The app's single in-memory store. Commands lock it briefly per call.
pub struct AppState(pub Mutex<Store>);

/// Staff PIN check and session (client notes, mark paid, close app).
pub struct StaffGate(pub Mutex<StaffAuth>);

/// The staff PIN comes from the `HH_STAFF_PIN` environment variable (4–8 digits),
/// so it's never in the code or the repo. Dev builds fall back to "0000" for
/// testing; release builds with no PIN set have no staff area.
fn staff_pin() -> Option<String> {
    std::env::var("HH_STAFF_PIN").ok().or_else(|| cfg!(debug_assertions).then(|| "0000".to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut store = Store::seed();
    // POC only: sample clients and notes so the staff area has something to show.
    store.add_sample_data(chrono::Local::now().naive_local());

    tauri::Builder::default()
        .manage(AppState(Mutex::new(store)))
        .manage(StaffGate(Mutex::new(StaffAuth::new(staff_pin()))))
        .invoke_handler(tauri::generate_handler![
            commands::list_services,
            commands::day_slots,
            commands::month_days,
            commands::create_booking,
            commands::staff_pin_configured,
            commands::staff_sign_in,
            commands::staff_sign_out,
            commands::staff_close_app,
            commands::staff_clients,
            commands::staff_client,
            commands::staff_add_note,
            commands::staff_edit_note,
            commands::staff_delete_note,
            commands::staff_mark_paid,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Healing Hideaway kiosk");
}
