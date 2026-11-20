use std::collections::HashMap;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use uuid::Uuid;

use crate::availability::{Schedule, month_availability, slots_for_day};
use crate::domain::*;
use crate::error::{BookingError, Result};

/// In-memory data for the POC. The app wraps it in a `Mutex`; later it moves
/// behind the shared API with SQL Server.
pub struct Store {
    pub services: Vec<Service>,
    pub schedule: Schedule,
    pub bookings: HashMap<Uuid, Booking>,
}

impl Store {
    pub fn seed() -> Self {
        // PLACEHOLDER prices and lengths: replace with Lindsey's real menu.
        // Lengths must be 30, 60 or 90 minutes. Prices are $0 so no invented
        // prices ship by accident.
        let svc = |id, name: &str, desc: &str, opts: &[(u32, i64)]| Service {
            id,
            name: name.into(),
            description: desc.into(),
            options: opts
                .iter()
                .map(|&(minutes, price_cents)| ServiceOption { minutes, price_cents })
                .collect(),
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
            .map(|b| (b.start, b.start + Duration::minutes(i64::from(b.minutes))))
            .collect()
    }

    pub fn day_slots(&self, date: NaiveDate, minutes: u32, now: NaiveDateTime) -> Vec<Slot> {
        slots_for_day(date, minutes, &self.schedule, &self.busy(), now)
    }

    pub fn month_days(&self, year: i32, month: u32, minutes: u32, now: NaiveDateTime) -> Vec<DayAvailability> {
        month_availability(year, month, minutes, &self.schedule, &self.busy(), now)
    }

    /// Confirms a booking immediately, as Unpaid. Re-checks availability here:
    /// never trust the screen's view of which slots are open.
    pub fn create_booking(
        &mut self,
        service_id: u32,
        minutes: u32,
        date: NaiveDate,
        start: &str,
        client: ClientDetails,
        now: NaiveDateTime,
    ) -> Result<BookingConfirmation> {
        let (service_name, price_cents) = self
            .services
            .iter()
            .find(|s| s.id == service_id)
            .and_then(|s| {
                s.options
                    .iter()
                    .find(|o| o.minutes == minutes)
                    .map(|o| (s.name.clone(), o.price_cents))
            })
            .ok_or(BookingError::UnknownService)?;

        let time = NaiveTime::parse_from_str(start, "%H:%M").map_err(|_| BookingError::InvalidTime)?;
        let slot = self
            .day_slots(date, minutes, now)
            .into_iter()
            .find(|s| s.start == start && s.status == SlotStatus::Open)
            .ok_or(BookingError::SlotTaken)?;

        let id = Uuid::new_v4();
        self.bookings.insert(
            id,
            Booking {
                id,
                service_id,
                minutes,
                start: date.and_time(time),
                price_cents,
                client,
                payment: PaymentStatus::Unpaid,
                paid_at: None,
            },
        );
        Ok(BookingConfirmation { id, service_name, minutes, date, start_label: slot.label, price_cents })
    }

    /// Staff action after taking payment on the mobile device.
    /// Idempotent: marking a paid booking again keeps the first `paid_at`.
    pub fn mark_paid(&mut self, id: Uuid, at: NaiveDateTime) -> Result<()> {
        let b = self.bookings.get_mut(&id).ok_or(BookingError::UnknownBooking)?;
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

    fn monday() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 23).unwrap()
    }

    fn dt(h: u32, m: u32) -> NaiveDateTime {
        monday().and_hms_opt(h, m, 0).unwrap()
    }

    fn now() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 20).unwrap().and_hms_opt(9, 0, 0).unwrap()
    }

    fn client() -> ClientDetails {
        ClientDetails { name: "Test Client".into(), phone: "555-0100".into(), email: String::new() }
    }

    fn book(s: &mut Store, minutes: u32, start: &str) -> Result<BookingConfirmation> {
        s.create_booking(1, minutes, monday(), start, client(), now())
    }

    #[test]
    fn booking_is_confirmed_unpaid_and_holds_its_slot() {
        let mut s = Store::seed();
        let c = book(&mut s, 60, "09:00").unwrap();
        assert_eq!(c.service_name, "Swedish");
        assert_eq!(c.start_label, "9:00 AM");
        assert_eq!(s.bookings[&c.id].payment, PaymentStatus::Unpaid);
        assert_eq!(s.busy(), vec![(dt(9, 0), dt(10, 0))]);
    }

    #[test]
    fn same_slot_cannot_be_booked_twice() {
        let mut s = Store::seed();
        book(&mut s, 60, "09:00").unwrap();
        assert_eq!(book(&mut s, 60, "09:00").unwrap_err(), BookingError::SlotTaken);
        assert_eq!(book(&mut s, 30, "09:30").unwrap_err(), BookingError::SlotTaken); // overlaps
        assert_eq!(s.bookings.len(), 1);
    }

    #[test]
    fn rejects_unknown_service_or_length() {
        let mut s = Store::seed();
        assert_eq!(s.create_booking(99, 60, monday(), "09:00", client(), now()).unwrap_err(), BookingError::UnknownService);
        // Hot stone is 90 minutes only
        assert_eq!(s.create_booking(3, 60, monday(), "09:00", client(), now()).unwrap_err(), BookingError::UnknownService);
    }

    #[test]
    fn rejects_off_grid_closed_and_malformed_times() {
        let mut s = Store::seed();
        assert_eq!(book(&mut s, 60, "09:15").unwrap_err(), BookingError::SlotTaken); // not a start time
        assert_eq!(book(&mut s, 60, "12:00").unwrap_err(), BookingError::SlotTaken); // lunch
        assert_eq!(book(&mut s, 60, "9am").unwrap_err(), BookingError::InvalidTime);
        let tuesday = NaiveDate::from_ymd_opt(2026, 11, 24).unwrap();
        assert_eq!(s.create_booking(1, 60, tuesday, "09:00", client(), now()).unwrap_err(), BookingError::SlotTaken);
    }

    #[test]
    fn ui_responses_reveal_no_client_details() {
        let mut s = Store::seed();
        let confirmation = book(&mut s, 60, "09:00").unwrap();
        let json = [
            serde_json::to_string(&s.day_slots(monday(), 60, now())).unwrap(),
            serde_json::to_string(&s.month_days(2026, 11, 60, now())).unwrap(),
            serde_json::to_string(&confirmation).unwrap(),
        ]
        .concat();
        assert!(!json.contains("Test Client"));
        assert!(!json.contains("555-0100"));
        assert!(json.contains(r#""startLabel":"9:00 AM""#)); // camelCase for the UI
        assert!(json.contains(r#""status":"unavailable""#));
    }

    #[test]
    fn mark_paid_is_idempotent() {
        let mut s = Store::seed();
        let id = book(&mut s, 60, "09:00").unwrap().id;
        s.mark_paid(id, dt(10, 0)).unwrap();
        s.mark_paid(id, dt(11, 0)).unwrap();
        assert_eq!(s.bookings[&id].payment, PaymentStatus::Paid);
        assert_eq!(s.bookings[&id].paid_at, Some(dt(10, 0)));
    }

    #[test]
    fn mark_paid_rejects_unknown_booking() {
        let mut s = Store::seed();
        assert_eq!(s.mark_paid(Uuid::new_v4(), dt(10, 0)).unwrap_err(), BookingError::UnknownBooking);
    }

    #[test]
    fn every_service_length_is_30_60_or_90() {
        for svc in Store::seed().services {
            for o in svc.options {
                assert!(matches!(o.minutes, 30 | 60 | 90), "{} has {} min", svc.name, o.minutes);
            }
        }
    }
}
