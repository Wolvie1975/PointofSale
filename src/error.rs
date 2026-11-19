use thiserror::Error;

use crate::domain::ProductId;

#[derive(Debug, Error)]
pub enum PosError {
    #[error("product {0} not found")]
    ProductNotFound(ProductId),

    #[error("insufficient stock for product {id}: requested {requested}, available {available}")]
    InsufficientStock {
        id: ProductId,
        requested: u32,
        available: u32,
    },

    #[error("a sale must contain at least one item")]
    EmptySale,

    #[error("quantity must be greater than zero")]
    InvalidQuantity,

    /// Backend failure. Once the SQL Server store lands, add variants such as
    /// `Database(#[from] tiberius::error::Error)` and a pool error instead of
    /// stringifying.
    #[error("storage error: {0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, PosError>;
