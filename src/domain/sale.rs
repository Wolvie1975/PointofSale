use super::{Money, ProductId};

pub type SaleId = i64;

/// One line on a receipt. Name and unit price are snapshotted at sale time so
/// later product edits don't rewrite sales history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineItem {
    pub product_id: ProductId,
    pub name: String,
    pub unit_price: Money,
    pub quantity: u32,
}

impl LineItem {
    pub fn total(&self) -> Money {
        self.unit_price * self.quantity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sale {
    pub id: SaleId,
    pub items: Vec<LineItem>,
    pub total: Money,
}

/// A priced sale ready to be committed by the store.
#[derive(Debug, Clone)]
pub struct NewSale {
    pub items: Vec<LineItem>,
}

impl NewSale {
    pub fn total(&self) -> Money {
        self.items.iter().map(LineItem::total).sum()
    }
}
