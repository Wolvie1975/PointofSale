use thiserror::Error;

/// Messages are shown on the kiosk (to clients or staff), so keep them plain.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoreError {
    #[error("Unknown service or length")]
    UnknownService,
    #[error("That time was just taken. Please pick another.")]
    SlotTaken,
    #[error("Invalid start time")]
    InvalidTime,
    #[error("Unknown booking")]
    UnknownBooking,
    #[error("Unknown client")]
    UnknownClient,
    #[error("Unknown note")]
    UnknownNote,
    #[error("The note is empty.")]
    EmptyNote,
    #[error("The note is too long (limit {limit} characters).")]
    NoteTooLong { limit: usize },
}

pub type Result<T> = std::result::Result<T, StoreError>;
