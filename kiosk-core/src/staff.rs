//! Staff PIN gate for staff-only actions (for now: closing the app).
//!
//! The PIN is checked here in Rust, never in the UI, and wrong guesses are
//! rate-limited: after `MAX_FAILURES` wrong PINs in a row, every attempt is
//! refused for `LOCKOUT` (even a correct one).

use chrono::{Duration, NaiveDateTime};
use thiserror::Error;

pub const MAX_FAILURES: u32 = 5;
pub const LOCKOUT: Duration = Duration::seconds(60);

/// Messages are shown to staff on the kiosk.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PinError {
    #[error("No staff PIN is set up on this machine.")]
    NotConfigured,
    #[error("Wrong PIN. {remaining} tries left.")]
    Wrong { remaining: u32 },
    #[error("Too many wrong tries. Try again in {seconds} seconds.")]
    Locked { seconds: i64 },
}

#[derive(Debug)]
pub struct PinGate {
    pin: Option<String>,
    failures: u32,
    locked_until: Option<NaiveDateTime>,
}

impl PinGate {
    /// `pin` must be 4–8 digits; anything else counts as not configured.
    pub fn new(pin: Option<String>) -> Self {
        let pin = pin
            .map(|p| p.trim().to_string())
            .filter(|p| (4..=8).contains(&p.len()) && p.chars().all(|c| c.is_ascii_digit()));
        Self { pin, failures: 0, locked_until: None }
    }

    pub fn is_configured(&self) -> bool {
        self.pin.is_some()
    }

    pub fn check(&mut self, attempt: &str, now: NaiveDateTime) -> Result<(), PinError> {
        let Some(pin) = &self.pin else { return Err(PinError::NotConfigured) };

        if let Some(until) = self.locked_until {
            if now < until {
                // Round up so it never says "0 seconds"
                let ms = (until - now).num_milliseconds();
                return Err(PinError::Locked { seconds: (ms + 999) / 1000 });
            }
            self.locked_until = None;
            self.failures = 0;
        }

        if constant_time_eq(pin.as_bytes(), attempt.as_bytes()) {
            self.failures = 0;
            return Ok(());
        }

        self.failures += 1;
        if self.failures >= MAX_FAILURES {
            self.locked_until = Some(now + LOCKOUT);
            return Err(PinError::Locked { seconds: LOCKOUT.num_seconds() });
        }
        Err(PinError::Wrong { remaining: MAX_FAILURES - self.failures })
    }
}

/// Compares without stopping at the first wrong digit, so timing doesn't leak
/// how much of the PIN was right.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn t(sec: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 20).unwrap().and_hms_opt(9, 0, sec).unwrap()
    }

    fn gate() -> PinGate {
        PinGate::new(Some("2468".into()))
    }

    #[test]
    fn correct_pin_passes() {
        assert_eq!(gate().check("2468", t(0)), Ok(()));
    }

    #[test]
    fn wrong_pin_counts_down() {
        let mut g = gate();
        assert_eq!(g.check("1111", t(0)), Err(PinError::Wrong { remaining: 4 }));
        assert_eq!(g.check("1111", t(1)), Err(PinError::Wrong { remaining: 3 }));
    }

    #[test]
    fn locks_after_five_wrong_even_for_the_right_pin() {
        let mut g = gate();
        for i in 0..4 {
            g.check("0000", t(i)).unwrap_err();
        }
        assert_eq!(g.check("0000", t(4)), Err(PinError::Locked { seconds: 60 }));
        assert_eq!(g.check("2468", t(10)), Err(PinError::Locked { seconds: 54 }));
    }

    #[test]
    fn lock_expires_and_resets_the_count() {
        let mut g = gate();
        for i in 0..5 {
            g.check("0000", t(i)).unwrap_err();
        }
        let later = t(4) + LOCKOUT;
        assert_eq!(g.check("0000", later), Err(PinError::Wrong { remaining: 4 }));
        assert_eq!(g.check("2468", later), Ok(()));
    }

    #[test]
    fn success_resets_the_count() {
        let mut g = gate();
        g.check("0000", t(0)).unwrap_err();
        g.check("2468", t(1)).unwrap();
        assert_eq!(g.check("0000", t(2)), Err(PinError::Wrong { remaining: 4 }));
    }

    #[test]
    fn missing_or_malformed_pin_is_not_configured() {
        for pin in [None, Some(""), Some("12"), Some("123456789"), Some("12ab")] {
            let mut g = PinGate::new(pin.map(String::from));
            assert!(!g.is_configured());
            assert_eq!(g.check("1234", t(0)), Err(PinError::NotConfigured));
        }
    }

    #[test]
    fn pin_is_trimmed() {
        assert!(PinGate::new(Some(" 2468\n".into())).check("2468", t(0)).is_ok());
    }
}
