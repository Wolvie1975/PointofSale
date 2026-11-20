//! Healing Hideaway kiosk booking logic.
//!
//! - [`domain`]: shared types, serialized to the UI in camelCase.
//! - [`availability`]: Lindsey's schedule and slot math.
//! - [`store`]: in-memory data plus the booking rules.
//!
//! Nothing here reads the clock: callers pass `now`, so every rule is testable.

pub mod availability;
pub mod domain;
pub mod error;
pub mod store;

pub use error::{BookingError, Result};
