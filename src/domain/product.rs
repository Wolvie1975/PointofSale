use super::Money;

/// `i64` so it maps cleanly to a SQL `BIGINT` / identity primary key.
pub type ProductId = i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    pub id: ProductId,
    pub sku: String,
    pub name: String,
    pub price: Money,
    pub stock: u32,
}

/// A product that has not been persisted yet (the store assigns the id).
#[derive(Debug, Clone)]
pub struct NewProduct {
    pub sku: String,
    pub name: String,
    pub price: Money,
    pub stock: u32,
}
