//! Point-of-sale core library.
//!
//! Layers:
//! - [`domain`]: plain data types (products, sales, money). No I/O.
//! - [`store`]: the persistence boundary (`PosStore` trait) plus implementations.
//! - [`service`]: business logic, generic over any `PosStore`.

pub mod domain;
pub mod error;
pub mod service;
pub mod store;

pub use error::{PosError, Result};
