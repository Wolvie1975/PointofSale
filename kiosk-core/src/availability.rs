use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Weekday};

use crate::domain::{DayAvailability, DayStatus, Slot, SlotStatus};

#[derive(Clone, Debug)]
pub struct DayHours {
    pub open: NaiveTime,
    pub close: NaiveTime,
    pub lunch: Option<(NaiveTime, NaiveTime)>,
}

#[derive(Clone, Debug)]
pub struct Schedule {
    pub weekly: HashMap<Weekday, DayHours>,          // missing day = closed
    pub buffer: Duration,                            // room reset between clients
    pub step: Duration,                              // spacing between start times
    pub min_notice: Duration,                        // how far ahead a slot must be booked
    pub blocks: Vec<(NaiveDateTime, NaiveDateTime)>, // time off / personal blocks
}

impl Schedule {
    /// Mon, Wed, Sat, Sun. Hours and lunch are PLACEHOLDERS: replace with Lindsey's.
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

/// Every candidate start time for `date`, marked Open or Unavailable.
/// `busy` = existing bookings (start, end) that hold their slot.
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
    let len = Duration::minutes(i64::from(minutes));
    let close = date.and_time(hours.close);
    let mut start = date.and_time(hours.open);
    let mut out = Vec::new();

    while start + len <= close {
        let end = start + len;
        let lunch = hours
            .lunch
            .is_some_and(|(a, b)| overlaps((start, end), (date.and_time(a), date.and_time(b))));
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
        let Some(next) = d.succ_opt() else { break };
        d = next;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, m, d).unwrap()
    }

    fn dt(m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
        date(m, d).and_hms_opt(h, min, 0).unwrap()
    }

    /// Far enough before every test date that notice never matters.
    fn early() -> NaiveDateTime {
        dt(11, 1, 8, 0)
    }

    fn status_at(slots: &[Slot], start: &str) -> SlotStatus {
        slots.iter().find(|s| s.start == start).unwrap_or_else(|| panic!("no {start} slot")).status
    }

    #[test]
    fn tuesday_is_closed() {
        let s = Schedule::healing_hideaway_default();
        assert!(slots_for_day(date(11, 24), 60, &s, &[], early()).is_empty());
    }

    #[test]
    fn starts_are_on_a_30_minute_grid() {
        let s = Schedule::healing_hideaway_default();
        let slots = slots_for_day(date(11, 23), 60, &s, &[], early()); // Mon 9:00–19:00
        assert_eq!(slots.first().unwrap().start, "09:00");
        assert_eq!(slots[1].start, "09:30");
        assert_eq!(slots.last().unwrap().start, "18:00"); // 18:00 + 60 = close
        assert_eq!(slots.len(), 19);
        assert_eq!(slots[0].label, "9:00 AM");
    }

    #[test]
    fn ninety_minute_service_must_end_by_close() {
        let s = Schedule::healing_hideaway_default();
        let slots = slots_for_day(date(11, 21), 90, &s, &[], early()); // Sat closes 17:00
        assert_eq!(slots.last().unwrap().start, "15:30");
    }

    #[test]
    fn booked_slot_is_unavailable() {
        let s = Schedule::healing_hideaway_default();
        let busy = [(dt(11, 21, 10, 30), dt(11, 21, 11, 30))];
        let slots = slots_for_day(date(11, 21), 60, &s, &busy, early());
        assert_eq!(status_at(&slots, "10:30"), SlotStatus::Unavailable);
    }

    #[test]
    fn buffer_pushes_next_start_to_the_half_hour() {
        let s = Schedule::healing_hideaway_default();
        let busy = [(dt(11, 23, 9, 0), dt(11, 23, 10, 0))]; // 60 min at 9:00
        let slots = slots_for_day(date(11, 23), 60, &s, &busy, early());
        assert_eq!(status_at(&slots, "10:00"), SlotStatus::Unavailable); // inside the 15 min reset
        assert_eq!(status_at(&slots, "10:30"), SlotStatus::Open);
    }

    #[test]
    fn buffer_applies_before_a_booking_too() {
        let s = Schedule::healing_hideaway_default();
        let busy = [(dt(11, 23, 14, 0), dt(11, 23, 15, 0))];
        let slots = slots_for_day(date(11, 23), 60, &s, &busy, early());
        assert_eq!(status_at(&slots, "13:00"), SlotStatus::Unavailable); // ends 14:00, no reset time
        assert_eq!(status_at(&slots, "12:30"), SlotStatus::Open); // ends 13:30
    }

    #[test]
    fn lunch_blocks_noon_and_overlapping_long_sessions() {
        let s = Schedule::healing_hideaway_default();
        let slots = slots_for_day(date(11, 23), 90, &s, &[], early());
        assert_eq!(status_at(&slots, "12:00"), SlotStatus::Unavailable);
        assert_eq!(status_at(&slots, "11:00"), SlotStatus::Unavailable); // 11:00–12:30
        assert_eq!(status_at(&slots, "10:30"), SlotStatus::Open); // ends exactly at noon
        assert_eq!(status_at(&slots, "12:30"), SlotStatus::Open);
    }

    #[test]
    fn needs_24_hours_notice() {
        let s = Schedule::healing_hideaway_default();
        let now = dt(11, 22, 10, 0); // Sunday 10:00
        let slots = slots_for_day(date(11, 23), 60, &s, &[], now);
        assert_eq!(status_at(&slots, "09:30"), SlotStatus::Unavailable);
        assert_eq!(status_at(&slots, "10:00"), SlotStatus::Open);
    }

    #[test]
    fn time_off_blocks_slots() {
        let mut s = Schedule::healing_hideaway_default();
        s.blocks.push((dt(11, 23, 15, 0), dt(11, 23, 17, 0)));
        let slots = slots_for_day(date(11, 23), 60, &s, &[], early());
        assert_eq!(status_at(&slots, "14:30"), SlotStatus::Unavailable);
        assert_eq!(status_at(&slots, "16:30"), SlotStatus::Unavailable);
        assert_eq!(status_at(&slots, "17:00"), SlotStatus::Open);
    }

    #[test]
    fn month_view_statuses() {
        let mut s = Schedule::healing_hideaway_default();
        // Sat 21: blocked all day → Full
        s.blocks.push((dt(11, 21, 0, 0), dt(11, 22, 0, 0)));
        // Sun 22 (11:00–17:00): only 15:00 and 15:30 left for 90 min → Limited
        s.blocks.push((dt(11, 22, 11, 0), dt(11, 22, 15, 0)));
        let now = dt(11, 19, 8, 0); // Thursday
        let days = month_availability(2026, 11, 90, &s, &[], now);
        let status = |d: u32| days[(d - 1) as usize].status;

        assert_eq!(days.len(), 30);
        assert_eq!(status(18), DayStatus::Past);
        assert_eq!(status(19), DayStatus::Closed); // Thu
        assert_eq!(status(20), DayStatus::Closed); // Fri
        assert_eq!(status(21), DayStatus::Full);
        assert_eq!(status(22), DayStatus::Limited);
        assert_eq!(status(23), DayStatus::Open); // Mon
        assert_eq!(status(24), DayStatus::Closed); // Tue
    }

    #[test]
    fn invalid_month_is_empty() {
        let s = Schedule::healing_hideaway_default();
        assert!(month_availability(2026, 13, 60, &s, &[], early()).is_empty());
    }
}
