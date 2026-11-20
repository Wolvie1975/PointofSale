use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceOption {
    pub minutes: u32, // 30, 60 or 90
    pub price_cents: i64,
}

#[derive(Clone, Debug, Serialize)]
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
    pub client: ClientDetails, // never sent to slot queries
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
