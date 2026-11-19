use crate::domain::{LineItem, NewProduct, NewSale, Product, ProductId, Sale, SaleId};
use crate::store::PosStore;
use crate::{PosError, Result};

/// Business logic. Generic over the store so the same code runs against the
/// in-memory store (POC/tests) and, later, the SQL store.
pub struct PosService<S> {
    store: S,
}

impl<S: PosStore> PosService<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn add_product(&self, product: NewProduct) -> Result<Product> {
        self.store.create_product(product).await
    }

    pub async fn list_products(&self) -> Result<Vec<Product>> {
        self.store.list_products().await
    }

    pub async fn get_sale(&self, id: SaleId) -> Result<Option<Sale>> {
        self.store.get_sale(id).await
    }

    /// Price a cart of `(product_id, quantity)` pairs and commit it as a sale.
    pub async fn checkout(&self, cart: &[(ProductId, u32)]) -> Result<Sale> {
        if cart.is_empty() {
            return Err(PosError::EmptySale);
        }

        let mut items = Vec::with_capacity(cart.len());
        for &(product_id, quantity) in cart {
            if quantity == 0 {
                return Err(PosError::InvalidQuantity);
            }
            let product = self
                .store
                .get_product(product_id)
                .await?
                .ok_or(PosError::ProductNotFound(product_id))?;
            items.push(LineItem {
                product_id,
                name: product.name,
                unit_price: product.price,
                quantity,
            });
        }

        self.store.commit_sale(NewSale { items }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Money;
    use crate::store::memory::InMemoryStore;

    async fn service_with_coffee(stock: u32) -> (PosService<InMemoryStore>, ProductId) {
        let svc = PosService::new(InMemoryStore::default());
        let p = svc
            .add_product(NewProduct {
                sku: "COF-001".into(),
                name: "Coffee".into(),
                price: Money::from_cents(350),
                stock,
            })
            .await
            .unwrap();
        (svc, p.id)
    }

    #[tokio::test]
    async fn checkout_totals_and_decrements_stock() {
        let (svc, id) = service_with_coffee(10).await;
        let sale = svc.checkout(&[(id, 3)]).await.unwrap();

        assert_eq!(sale.total, Money::from_cents(1050));
        assert_eq!(svc.list_products().await.unwrap()[0].stock, 7);
        assert_eq!(svc.get_sale(sale.id).await.unwrap(), Some(sale));
    }

    #[tokio::test]
    async fn insufficient_stock_leaves_state_untouched() {
        let (svc, id) = service_with_coffee(2).await;
        // Two lines that together exceed stock.
        let err = svc.checkout(&[(id, 1), (id, 2)]).await.unwrap_err();

        assert!(matches!(err, PosError::InsufficientStock { requested: 3, available: 2, .. }));
        assert_eq!(svc.list_products().await.unwrap()[0].stock, 2);
    }

    #[tokio::test]
    async fn rejects_bad_carts() {
        let (svc, id) = service_with_coffee(5).await;
        assert!(matches!(svc.checkout(&[]).await, Err(PosError::EmptySale)));
        assert!(matches!(svc.checkout(&[(id, 0)]).await, Err(PosError::InvalidQuantity)));
        assert!(matches!(svc.checkout(&[(999, 1)]).await, Err(PosError::ProductNotFound(999))));
    }
}
