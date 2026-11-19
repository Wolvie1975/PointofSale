use pos::domain::{Money, NewProduct};
use pos::service::PosService;
use pos::store::memory::InMemoryStore;

#[tokio::main]
async fn main() -> pos::Result<()> {
    // POC: in-memory store. When SQL Server persistence lands, this becomes:
    //   let store = MssqlStore::connect(&database_url).await?;
    let pos = PosService::new(InMemoryStore::default());

    let coffee = pos
        .add_product(NewProduct {
            sku: "COF-001".into(),
            name: "Coffee".into(),
            price: Money::from_cents(350),
            stock: 50,
        })
        .await?;
    let bagel = pos
        .add_product(NewProduct {
            sku: "BAG-001".into(),
            name: "Bagel".into(),
            price: Money::from_cents(275),
            stock: 20,
        })
        .await?;

    let sale = pos.checkout(&[(coffee.id, 2), (bagel.id, 1)]).await?;

    println!("Sale #{}", sale.id);
    for item in &sale.items {
        println!(
            "  {:<10} x{:<3} @ {:>7} = {:>8}",
            item.name,
            item.quantity,
            item.unit_price.to_string(),
            item.total().to_string()
        );
    }
    println!("  {:>36}", format!("Total: {}", sale.total));

    println!("\nInventory:");
    for p in pos.list_products().await? {
        println!("  {} {:<10} {:>3} left", p.sku, p.name, p.stock);
    }

    Ok(())
}
