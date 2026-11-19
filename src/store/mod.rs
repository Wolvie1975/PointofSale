//! Persistence boundary.
//!
//! The POC ships only [`memory::InMemoryStore`]. The planned SQL Server backend
//! will live in `store/mssql.rs` as `MssqlStore` and implement the same trait, so the
//! service layer and `main` change by one line when it lands.
//!
//! Methods are async now (even though the in-memory store never awaits) so the
//! SQL implementation can drop in without changing any call sites.

pub mod memory;

use std::future::Future;

use crate::Result;
use crate::domain::{NewProduct, NewSale, Product, ProductId, Sale, SaleId};

pub trait PosStore: Send + Sync {
    fn create_product(&self, product: NewProduct) -> impl Future<Output = Result<Product>> + Send;

    fn get_product(&self, id: ProductId) -> impl Future<Output = Result<Option<Product>>> + Send;

    fn list_products(&self) -> impl Future<Output = Result<Vec<Product>>> + Send;

    /// Atomically verify stock, decrement it, and record the sale.
    /// Either everything is applied or nothing is (SQL: one transaction).
    fn commit_sale(&self, sale: NewSale) -> impl Future<Output = Result<Sale>> + Send;

    fn get_sale(&self, id: SaleId) -> impl Future<Output = Result<Option<Sale>>> + Send;
}
