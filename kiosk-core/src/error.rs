use thiserror::Error;

/// Messages are shown to clients on the kiosk, so keep them plain.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BookingError {
    #[error("Unknown service or length")]
    UnknownService,
    #[error("That time was just taken. Please pick another.")]
    SlotTaken,
    #[error("Invalid start time")]
    InvalidTime,
    #[error("Unknown booking")]
    UnknownBooking,
}

pub type Result<T> = std::result::Result<T, BookingError>;
