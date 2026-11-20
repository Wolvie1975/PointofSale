use std::cmp::Reverse;
use std::collections::HashMap;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use uuid::Uuid;

use crate::availability::{Schedule, month_availability, slots_for_day};
use crate::domain::*;
use crate::error::{StoreError, Result};

/// In-memory data for the POC. The app wraps it in a `Mutex`; later it moves
/// behind the shared API with SQL Server.
pub struct Store {
    pub services: Vec<Service>,
    pub schedule: Schedule,
    pub bookings: HashMap<Uuid, Booking>,
    pub clients: HashMap<Uuid, Client>,
    pub notes: HashMap<Uuid, Note>,
}

/// Longest note staff can save.
pub const NOTE_LIMIT: usize = 4000;

fn digits(s: &str) -> String {
    s.chars().filter(char::is_ascii_digit).collect()
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
            clients: HashMap::new(),
            notes: HashMap::new(),
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
            .ok_or(StoreError::UnknownService)?;

        let time = NaiveTime::parse_from_str(start, "%H:%M").map_err(|_| StoreError::InvalidTime)?;
        let slot = self
            .day_slots(date, minutes, now)
            .into_iter()
            .find(|s| s.start == start && s.status == SlotStatus::Open)
            .ok_or(StoreError::SlotTaken)?;

        let client_id = self.find_or_create_client(client);
        let id = Uuid::new_v4();
        self.bookings.insert(
            id,
            Booking {
                id,
                service_id,
                minutes,
                start: date.and_time(time),
                price_cents,
                client_id,
                payment: PaymentStatus::Unpaid,
                paid_at: None,
            },
        );
        Ok(BookingConfirmation { id, service_name, minutes, date, start_label: slot.label, price_cents })
    }

    /// Staff action after taking payment on the mobile device.
    /// Idempotent: marking a paid booking again keeps the first `paid_at`.
    pub fn mark_paid(&mut self, id: Uuid, at: NaiveDateTime) -> Result<()> {
        let b = self.bookings.get_mut(&id).ok_or(StoreError::UnknownBooking)?;
        if b.payment == PaymentStatus::Unpaid {
            b.payment = PaymentStatus::Paid;
            b.paid_at = Some(at);
        }
        Ok(())
    }

    // ---- Clients ----

    /// The same person is matched by phone (digits only, 7+) or email (any case).
    /// A match takes their latest non-empty details.
    pub fn find_or_create_client(&mut self, details: ClientDetails) -> Uuid {
        let name = details.name.trim().to_string();
        let phone = details.phone.trim().to_string();
        let email = details.email.trim().to_string();
        let phone_key = digits(&phone);
        let email_key = email.to_lowercase();

        let existing = self.clients.values_mut().find(|c| {
            (phone_key.len() >= 7 && digits(&c.phone) == phone_key)
                || (email_key.contains('@') && c.email.to_lowercase() == email_key)
        });
        if let Some(c) = existing {
            if !name.is_empty() {
                c.name = name;
            }
            if !phone.is_empty() {
                c.phone = phone;
            }
            if !email.is_empty() {
                c.email = email;
            }
            return c.id;
        }
        let id = Uuid::new_v4();
        self.clients.insert(id, Client { id, name, phone, email });
        id
    }

    fn service_name(&self, service_id: u32) -> String {
        self.services
            .iter()
            .find(|s| s.id == service_id)
            .map_or_else(|| "Unknown service".into(), |s| s.name.clone())
    }

    /// Staff client list, sorted by name. `query` matches name, email or phone digits.
    pub fn list_clients(&self, query: &str, now: NaiveDateTime) -> Vec<ClientSummary> {
        let q = query.trim().to_lowercase();
        let q_digits = digits(&q);
        let mut out: Vec<ClientSummary> = self
            .clients
            .values()
            .filter(|c| {
                q.is_empty()
                    || c.name.to_lowercase().contains(&q)
                    || c.email.to_lowercase().contains(&q)
                    || (!q_digits.is_empty() && digits(&c.phone).contains(&q_digits))
            })
            .map(|c| {
                let bookings: Vec<&Booking> = self.bookings.values().filter(|b| b.client_id == c.id).collect();
                ClientSummary {
                    id: c.id,
                    name: c.name.clone(),
                    phone: c.phone.clone(),
                    email: c.email.clone(),
                    bookings: bookings.len(),
                    unpaid: bookings.iter().filter(|b| b.payment == PaymentStatus::Unpaid).count(),
                    notes: self.notes.values().filter(|n| n.client_id == c.id).count(),
                    next_visit: bookings.iter().map(|b| b.start).filter(|&t| t >= now).min(),
                }
            })
            .collect();
        out.sort_by_key(|c| c.name.to_lowercase());
        out
    }

    /// One client's page: details, bookings and private notes (newest first).
    pub fn client_record(&self, client_id: Uuid) -> Result<ClientRecord> {
        let client = self.clients.get(&client_id).ok_or(StoreError::UnknownClient)?.clone();
        let mut bookings: Vec<BookingView> = self
            .bookings
            .values()
            .filter(|b| b.client_id == client_id)
            .map(|b| BookingView {
                id: b.id,
                service_name: self.service_name(b.service_id),
                minutes: b.minutes,
                start: b.start,
                price_cents: b.price_cents,
                payment: b.payment,
                paid_at: b.paid_at,
            })
            .collect();
        bookings.sort_by_key(|b| Reverse(b.start));
        let mut notes: Vec<Note> = self.notes.values().filter(|n| n.client_id == client_id).cloned().collect();
        notes.sort_by_key(|n| Reverse(n.created_at));
        Ok(ClientRecord { client, bookings, notes })
    }

    // ---- Private notes (staff only) ----

    fn clean_note(text: &str) -> Result<String> {
        let text = text.trim();
        if text.is_empty() {
            return Err(StoreError::EmptyNote);
        }
        if text.chars().count() > NOTE_LIMIT {
            return Err(StoreError::NoteTooLong { limit: NOTE_LIMIT });
        }
        Ok(text.to_string())
    }

    pub fn add_note(&mut self, client_id: Uuid, text: &str, now: NaiveDateTime) -> Result<Note> {
        if !self.clients.contains_key(&client_id) {
            return Err(StoreError::UnknownClient);
        }
        let note = Note { id: Uuid::new_v4(), client_id, text: Self::clean_note(text)?, created_at: now, edited_at: None };
        self.notes.insert(note.id, note.clone());
        Ok(note)
    }

    pub fn edit_note(&mut self, note_id: Uuid, text: &str, now: NaiveDateTime) -> Result<Note> {
        let text = Self::clean_note(text)?;
        let note = self.notes.get_mut(&note_id).ok_or(StoreError::UnknownNote)?;
        if note.text != text {
            note.text = text;
            note.edited_at = Some(now);
        }
        Ok(note.clone())
    }

    pub fn delete_note(&mut self, note_id: Uuid) -> Result<()> {
        self.notes.remove(&note_id).map(|_| ()).ok_or(StoreError::UnknownNote)
    }

    /// POC only: a few obviously fake clients with past visits and notes, so the
    /// staff area has something to show. Past dates don't affect availability.
    pub fn add_sample_data(&mut self, now: NaiveDateTime) {
        let days_ago = |n: i64, h: u32| (now.date() - Duration::days(n)).and_hms_opt(h, 0, 0).unwrap();
        struct Sample {
            name: &'static str,
            phone: &'static str,
            email: &'static str,
            service_id: u32,
            minutes: u32,
            days_ago: i64,
            paid: bool,
            notes: &'static [&'static str],
        }
        let samples = [
            Sample {
                name: "Avery Example",
                phone: "555-0101",
                email: "avery@example.com",
                service_id: 1,
                minutes: 60,
                days_ago: 14,
                paid: true,
                notes: &[
                    "Prefers lighter pressure on the shoulders.",
                    "Mentioned lower back stiffness from desk work. Check in next visit.",
                ],
            },
            Sample {
                name: "Jordan Sample",
                phone: "555-0102",
                email: "",
                service_id: 2,
                minutes: 90,
                days_ago: 7,
                paid: false,
                notes: &["Allergic to nut oils: use the unscented, nut-free lotion."],
            },
            Sample {
                name: "Riley Test",
                phone: "",
                email: "riley@example.com",
                service_id: 3,
                minutes: 90,
                days_ago: 3,
                paid: true,
                notes: &[],
            },
        ];
        for (i, Sample { name, phone, email, service_id, minutes, days_ago: ago, paid, notes }) in
            samples.into_iter().enumerate()
        {
            let client_id = self.find_or_create_client(ClientDetails {
                name: name.into(),
                phone: phone.into(),
                email: email.into(),
            });
            let start = days_ago(ago, 10);
            let price_cents = self
                .services
                .iter()
                .find(|s| s.id == service_id)
                .and_then(|s| s.options.iter().find(|o| o.minutes == minutes))
                .map_or(0, |o| o.price_cents);
            let id = Uuid::new_v4();
            self.bookings.insert(id, Booking {
                id,
                service_id,
                minutes,
                start,
                price_cents,
                client_id,
                payment: if paid { PaymentStatus::Paid } else { PaymentStatus::Unpaid },
                paid_at: paid.then(|| start + Duration::minutes(i64::from(minutes))),
            });
            for (j, text) in notes.iter().enumerate() {
                let at = start + Duration::minutes(i64::from(minutes) + 5 + j as i64 + i as i64);
                self.add_note(client_id, text, at).expect("sample note is valid");
            }
        }
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
        assert_eq!(book(&mut s, 60, "09:00").unwrap_err(), StoreError::SlotTaken);
        assert_eq!(book(&mut s, 30, "09:30").unwrap_err(), StoreError::SlotTaken); // overlaps
        assert_eq!(s.bookings.len(), 1);
    }

    #[test]
    fn rejects_unknown_service_or_length() {
        let mut s = Store::seed();
        assert_eq!(s.create_booking(99, 60, monday(), "09:00", client(), now()).unwrap_err(), StoreError::UnknownService);
        // Hot stone is 90 minutes only
        assert_eq!(s.create_booking(3, 60, monday(), "09:00", client(), now()).unwrap_err(), StoreError::UnknownService);
    }

    #[test]
    fn rejects_off_grid_closed_and_malformed_times() {
        let mut s = Store::seed();
        assert_eq!(book(&mut s, 60, "09:15").unwrap_err(), StoreError::SlotTaken); // not a start time
        assert_eq!(book(&mut s, 60, "12:00").unwrap_err(), StoreError::SlotTaken); // lunch
        assert_eq!(book(&mut s, 60, "9am").unwrap_err(), StoreError::InvalidTime);
        let tuesday = NaiveDate::from_ymd_opt(2026, 11, 24).unwrap();
        assert_eq!(s.create_booking(1, 60, tuesday, "09:00", client(), now()).unwrap_err(), StoreError::SlotTaken);
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
        assert_eq!(s.mark_paid(Uuid::new_v4(), dt(10, 0)).unwrap_err(), StoreError::UnknownBooking);
    }

    #[test]
    fn every_service_length_is_30_60_or_90() {
        for svc in Store::seed().services {
            for o in svc.options {
                assert!(matches!(o.minutes, 30 | 60 | 90), "{} has {} min", svc.name, o.minutes);
            }
        }
    }

    // ---- Clients and private notes ----

    fn details(name: &str, phone: &str, email: &str) -> ClientDetails {
        ClientDetails { name: name.into(), phone: phone.into(), email: email.into() }
    }

    #[test]
    fn same_person_matched_by_phone_or_email() {
        let mut s = Store::seed();
        let a = s.find_or_create_client(details("Sam", "(555) 010-0200", ""));
        let b = s.find_or_create_client(details("Sam Lee", "555 010 0200", "sam@example.com"));
        let c = s.find_or_create_client(details("", "", "SAM@example.com"));
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(s.clients.len(), 1);
        let client = &s.clients[&a];
        assert_eq!(client.name, "Sam Lee"); // latest non-empty details win
        assert_eq!(client.email, "SAM@example.com");
    }

    #[test]
    fn different_people_stay_separate() {
        let mut s = Store::seed();
        let a = s.find_or_create_client(details("A", "123", ""));
        let b = s.find_or_create_client(details("B", "123", "")); // too short to match on
        let c = s.find_or_create_client(details("C", "555-0300", "c@example.com"));
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_eq!(s.clients.len(), 3);
    }

    #[test]
    fn booking_twice_links_to_one_client() {
        let mut s = Store::seed();
        let first = book(&mut s, 60, "09:00").unwrap().id;
        let second = book(&mut s, 60, "14:00").unwrap().id;
        assert_eq!(s.clients.len(), 1);
        assert_eq!(s.bookings[&first].client_id, s.bookings[&second].client_id);
    }

    #[test]
    fn client_list_searches_and_counts() {
        let mut s = Store::seed();
        let id = book(&mut s, 60, "09:00").unwrap().id;
        let client_id = s.bookings[&id].client_id;
        s.add_note(client_id, "Likes the room warm", now()).unwrap();
        s.find_or_create_client(details("Zed Other", "555-0999", "zed@example.com"));

        let all = s.list_clients("", now());
        assert_eq!(all.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["Test Client", "Zed Other"]);
        let tc = &all[0];
        assert_eq!((tc.bookings, tc.unpaid, tc.notes), (1, 1, 1));
        assert_eq!(tc.next_visit, Some(dt(9, 0)));

        assert_eq!(s.list_clients("test", now()).len(), 1);
        assert_eq!(s.list_clients("5550100", now())[0].name, "Test Client"); // phone digits
        assert_eq!(s.list_clients("ZED@", now())[0].name, "Zed Other");
        assert!(s.list_clients("nobody", now()).is_empty());
    }

    #[test]
    fn notes_add_edit_delete() {
        let mut s = Store::seed();
        let client_id = s.find_or_create_client(details("Sam", "555-0200", ""));
        let n1 = s.add_note(client_id, "  First  ", dt(9, 0)).unwrap();
        assert_eq!(n1.text, "First"); // trimmed
        let n2 = s.add_note(client_id, "Second", dt(10, 0)).unwrap();

        let rec = s.client_record(client_id).unwrap();
        assert_eq!(rec.notes.iter().map(|n| n.id).collect::<Vec<_>>(), [n2.id, n1.id]); // newest first

        assert_eq!(s.edit_note(n1.id, "First", dt(11, 0)).unwrap().edited_at, None); // unchanged text
        let edited = s.edit_note(n1.id, "First, updated", dt(11, 0)).unwrap();
        assert_eq!((edited.text.as_str(), edited.edited_at), ("First, updated", Some(dt(11, 0))));
        assert_eq!(edited.created_at, dt(9, 0));

        s.delete_note(n2.id).unwrap();
        assert_eq!(s.client_record(client_id).unwrap().notes.len(), 1);
        assert_eq!(s.delete_note(n2.id).unwrap_err(), StoreError::UnknownNote);
    }

    #[test]
    fn notes_are_validated() {
        let mut s = Store::seed();
        let client_id = s.find_or_create_client(details("Sam", "555-0200", ""));
        assert_eq!(s.add_note(client_id, "   ", now()).unwrap_err(), StoreError::EmptyNote);
        let long = "x".repeat(NOTE_LIMIT + 1);
        assert_eq!(s.add_note(client_id, &long, now()).unwrap_err(), StoreError::NoteTooLong { limit: NOTE_LIMIT });
        assert_eq!(s.add_note(Uuid::new_v4(), "hi", now()).unwrap_err(), StoreError::UnknownClient);
        let n = s.add_note(client_id, "ok", now()).unwrap();
        assert_eq!(s.edit_note(n.id, "", now()).unwrap_err(), StoreError::EmptyNote);
        assert_eq!(s.edit_note(Uuid::new_v4(), "hi", now()).unwrap_err(), StoreError::UnknownNote);
    }

    #[test]
    fn client_facing_responses_never_include_notes() {
        let mut s = Store::seed();
        let confirmation = book(&mut s, 60, "09:00").unwrap();
        let client_id = s.bookings[&confirmation.id].client_id;
        s.add_note(client_id, "SECRET-NOTE-TEXT", now()).unwrap();
        let services = s.services.clone();
        let json = [
            serde_json::to_string(&services).unwrap(),
            serde_json::to_string(&s.day_slots(monday(), 60, now())).unwrap(),
            serde_json::to_string(&s.month_days(2026, 11, 60, now())).unwrap(),
            serde_json::to_string(&confirmation).unwrap(),
        ]
        .concat();
        assert!(!json.contains("SECRET-NOTE-TEXT"));
        assert!(!json.contains(&client_id.to_string()));
    }

    #[test]
    fn mark_paid_shows_on_the_client_page() {
        let mut s = Store::seed();
        let id = book(&mut s, 60, "09:00").unwrap().id;
        let client_id = s.bookings[&id].client_id;
        s.mark_paid(id, dt(10, 0)).unwrap();
        let rec = s.client_record(client_id).unwrap();
        assert_eq!(rec.bookings[0].payment, PaymentStatus::Paid);
        assert_eq!(s.list_clients("", now())[0].unpaid, 0);
    }

    #[test]
    fn sample_data_is_in_the_past_and_has_notes() {
        let mut s = Store::seed();
        s.add_sample_data(now());
        assert_eq!(s.clients.len(), 3);
        assert_eq!(s.notes.len(), 3);
        assert!(s.bookings.values().all(|b| b.start < now()));
        // Future availability is untouched
        assert_eq!(s.day_slots(monday(), 60, now()), Store::seed().day_slots(monday(), 60, now()));
        let jordan = s.list_clients("jordan", now());
        assert_eq!((jordan[0].unpaid, jordan[0].notes), (1, 1));
    }
}
