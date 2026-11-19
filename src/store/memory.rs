use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use super::PosStore;
use crate::domain::{NewProduct, NewSale, Product, ProductId, Sale, SaleId};
use crate::{PosError, Result};

/// Volatile store for the POC and for tests. Data is lost on exit.
#[derive(Debug, Default)]
pub struct InMemoryStore {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    products: HashMap<ProductId, Product>,
    sales: HashMap<SaleId, Sale>,
    last_product_id: ProductId,
    last_sale_id: SaleId,
}

impl InMemoryStore {
    fn lock(&self) -> Result<MutexGuard<'_, Inner>> {
        self.inner
            .lock()
            .map_err(|_| PosError::Storage("in-memory store lock poisoned".into()))
    }
}

impl PosStore for InMemoryStore {
    async fn create_product(&self, new: NewProduct) -> Result<Product> {
        let mut inner = self.lock()?;
        inner.last_product_id += 1;
        let product = Product {
            id: inner.last_product_id,
            sku: new.sku,
            name: new.name,
            price: new.price,
            stock: new.stock,
        };
        inner.products.insert(product.id, product.clone());
        Ok(product)
    }

    async fn get_product(&self, id: ProductId) -> Result<Option<Product>> {
        Ok(self.lock()?.products.get(&id).cloned())
    }

    async fn list_products(&self) -> Result<Vec<Product>> {
        let mut products: Vec<_> = self.lock()?.products.values().cloned().collect();
        products.sort_by_key(|p| p.id);
        Ok(products)
    }

    async fn commit_sale(&self, sale: NewSale) -> Result<Sale> {
        let mut inner = self.lock()?;

        // Aggregate per product so duplicate lines are checked against total demand.
        let mut demand: HashMap<ProductId, u32> = HashMap::new();
        for item in &sale.items {
            *demand.entry(item.product_id).or_default() += item.quantity;
        }

        // Validate everything before mutating anything (all-or-nothing).
        for (&id, &requested) in &demand {
            let product = inner
                .products
                .get(&id)
                .ok_or(PosError::ProductNotFound(id))?;
            if product.stock < requested {
                return Err(PosError::InsufficientStock {
                    id,
                    requested,
                    available: product.stock,
                });
            }
        }

        for (id, requested) in demand {
            if let Some(product) = inner.products.get_mut(&id) {
                product.stock -= requested;
            }
        }

        inner.last_sale_id += 1;
        let total = sale.total();
        let sale = Sale {
            id: inner.last_sale_id,
            items: sale.items,
            total,
        };
        inner.sales.insert(sale.id, sale.clone());
        Ok(sale)
    }

    async fn get_sale(&self, id: SaleId) -> Result<Option<Sale>> {
        Ok(self.lock()?.sales.get(&id).cloned())
    }
}
