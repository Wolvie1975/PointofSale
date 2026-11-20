//! Staff sign-in for staff-only actions (client notes, mark paid, close app).
//!
//! The PIN is checked here in Rust, never in the UI, and wrong guesses are
//! rate-limited: after `MAX_FAILURES` wrong PINs in a row, every attempt is
//! refused for `LOCKOUT` (even a correct one). A correct PIN starts a session:
//! the UI gets a random token that every staff command must present, and the
//! session ends after `SESSION_IDLE` without staff activity.

use chrono::{Duration, NaiveDateTime};
use thiserror::Error;
use uuid::Uuid;

pub const MAX_FAILURES: u32 = 5;
pub const LOCKOUT: Duration = Duration::seconds(60);
pub const SESSION_IDLE: Duration = Duration::minutes(5);

/// Messages are shown to staff on the kiosk.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StaffError {
    #[error("No staff PIN is set up on this machine.")]
    NotConfigured,
    #[error("Wrong PIN. {remaining} tries left.")]
    Wrong { remaining: u32 },
    #[error("Too many wrong tries. Try again in {seconds} seconds.")]
    Locked { seconds: i64 },
    #[error("Staff session ended. Please sign in again.")]
    SignedOut,
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

    pub fn check(&mut self, attempt: &str, now: NaiveDateTime) -> Result<(), StaffError> {
        let Some(pin) = &self.pin else { return Err(StaffError::NotConfigured) };

        if let Some(until) = self.locked_until {
            if now < until {
                // Round up so it never says "0 seconds"
                let ms = (until - now).num_milliseconds();
                return Err(StaffError::Locked { seconds: (ms + 999) / 1000 });
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
            return Err(StaffError::Locked { seconds: LOCKOUT.num_seconds() });
        }
        Err(StaffError::Wrong { remaining: MAX_FAILURES - self.failures })
    }
}

/// PIN gate plus the single staff session (one screen, one staff member at a time).
#[derive(Debug)]
pub struct StaffAuth {
    gate: PinGate,
    session: Option<Session>,
}

#[derive(Debug)]
struct Session {
    token: String,
    expires: NaiveDateTime,
}

impl StaffAuth {
    pub fn new(pin: Option<String>) -> Self {
        Self { gate: PinGate::new(pin), session: None }
    }

    pub fn is_configured(&self) -> bool {
        self.gate.is_configured()
    }

    /// Checks the PIN and starts a fresh session (ending any previous one).
    pub fn sign_in(&mut self, pin: &str, now: NaiveDateTime) -> Result<String, StaffError> {
        self.gate.check(pin, now)?;
        let token = Uuid::new_v4().simple().to_string();
        self.session = Some(Session { token: token.clone(), expires: now + SESSION_IDLE });
        Ok(token)
    }

    /// Allows a staff action if `token` is the live session, and extends it.
    pub fn authorize(&mut self, token: &str, now: NaiveDateTime) -> Result<(), StaffError> {
        match &mut self.session {
            Some(s) if now >= s.expires => {
                self.session = None;
                Err(StaffError::SignedOut)
            }
            Some(s) if constant_time_eq(s.token.as_bytes(), token.as_bytes()) => {
                s.expires = now + SESSION_IDLE;
                Ok(())
            }
            _ => Err(StaffError::SignedOut),
        }
    }

    /// Ends the session if `token` is the live one.
    pub fn sign_out(&mut self, token: &str) {
        if self.session.as_ref().is_some_and(|s| constant_time_eq(s.token.as_bytes(), token.as_bytes())) {
            self.session = None;
        }
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
        assert_eq!(g.check("1111", t(0)), Err(StaffError::Wrong { remaining: 4 }));
        assert_eq!(g.check("1111", t(1)), Err(StaffError::Wrong { remaining: 3 }));
    }

    #[test]
    fn locks_after_five_wrong_even_for_the_right_pin() {
        let mut g = gate();
        for i in 0..4 {
            g.check("0000", t(i)).unwrap_err();
        }
        assert_eq!(g.check("0000", t(4)), Err(StaffError::Locked { seconds: 60 }));
        assert_eq!(g.check("2468", t(10)), Err(StaffError::Locked { seconds: 54 }));
    }

    #[test]
    fn lock_expires_and_resets_the_count() {
        let mut g = gate();
        for i in 0..5 {
            g.check("0000", t(i)).unwrap_err();
        }
        let later = t(4) + LOCKOUT;
        assert_eq!(g.check("0000", later), Err(StaffError::Wrong { remaining: 4 }));
        assert_eq!(g.check("2468", later), Ok(()));
    }

    #[test]
    fn success_resets_the_count() {
        let mut g = gate();
        g.check("0000", t(0)).unwrap_err();
        g.check("2468", t(1)).unwrap();
        assert_eq!(g.check("0000", t(2)), Err(StaffError::Wrong { remaining: 4 }));
    }

    #[test]
    fn missing_or_malformed_pin_is_not_configured() {
        for pin in [None, Some(""), Some("12"), Some("123456789"), Some("12ab")] {
            let mut g = PinGate::new(pin.map(String::from));
            assert!(!g.is_configured());
            assert_eq!(g.check("1234", t(0)), Err(StaffError::NotConfigured));
        }
    }

    fn auth() -> StaffAuth {
        StaffAuth::new(Some("2468".into()))
    }

    #[test]
    fn sign_in_gives_a_token_that_authorizes() {
        let mut a = auth();
        let token = a.sign_in("2468", t(0)).unwrap();
        assert_eq!(a.authorize(&token, t(1)), Ok(()));
        assert_eq!(a.authorize("not-the-token", t(1)), Err(StaffError::SignedOut));
        assert_eq!(a.authorize("", t(1)), Err(StaffError::SignedOut));
    }

    #[test]
    fn wrong_pin_gives_no_session() {
        let mut a = auth();
        assert_eq!(a.sign_in("1111", t(0)), Err(StaffError::Wrong { remaining: 4 }));
        assert_eq!(a.authorize("", t(1)), Err(StaffError::SignedOut));
    }

    #[test]
    fn session_ends_after_idle_and_activity_extends_it() {
        let mut a = auth();
        let token = a.sign_in("2468", t(0)).unwrap();
        let almost = t(0) + SESSION_IDLE - Duration::seconds(1);
        assert_eq!(a.authorize(&token, almost), Ok(())); // extends to almost + IDLE
        assert_eq!(a.authorize(&token, almost + SESSION_IDLE - Duration::seconds(1)), Ok(()));
        let idle = almost + SESSION_IDLE - Duration::seconds(1) + SESSION_IDLE;
        assert_eq!(a.authorize(&token, idle), Err(StaffError::SignedOut));
        assert_eq!(a.authorize(&token, idle), Err(StaffError::SignedOut)); // stays ended
    }

    #[test]
    fn sign_out_and_new_sign_in_end_the_old_session() {
        let mut a = auth();
        let first = a.sign_in("2468", t(0)).unwrap();
        let second = a.sign_in("2468", t(1)).unwrap();
        assert_ne!(first, second);
        assert_eq!(a.authorize(&first, t(2)), Err(StaffError::SignedOut));
        a.sign_out("wrong");
        assert_eq!(a.authorize(&second, t(3)), Ok(()));
        a.sign_out(&second);
        assert_eq!(a.authorize(&second, t(4)), Err(StaffError::SignedOut));
    }

    #[test]
    fn pin_is_trimmed() {
        assert!(PinGate::new(Some(" 2468\n".into())).check("2468", t(0)).is_ok());
    }
}
