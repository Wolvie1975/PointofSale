# PointofSale

Point-of-sale proof of concept in Rust. The POC keeps all data in memory.
Microsoft SQL Server persistence is planned (see below) but not part of this phase.

## Run

```sh
cargo run      # demo: creates products, rings up a sale, prints receipt + inventory
cargo test
```

## Layout

```
src/
  main.rs            demo binary; wires a store into the service
  lib.rs
  error.rs           PosError / Result
  domain/            plain data types, no I/O
    money.rs         Money as integer cents (i64)
    product.rs
    sale.rs
  store/
    mod.rs           PosStore trait: the persistence boundary
    memory.rs        InMemoryStore (POC + tests)
  service.rs         PosService<S: PosStore>: checkout and other business logic
```

## Persistence plan (post-POC): Microsoft SQL Server

**Driver:** [`tiberius`](https://crates.io/crates/tiberius) (async, pure-Rust TDS
client) with `bb8-tiberius` for connection pooling. `sqlx` is not an option
because it dropped SQL Server support in 0.7. The dependencies are listed,
commented out, in `Cargo.toml`.

These choices were made now so adding the database doesn't force a refactor:

- **Store trait.** Everything goes through `PosStore`. The service is generic
  over it, so adding SQL Server means writing one new implementation and
  changing one line in `main.rs`.
- **Async from day one.** Trait methods are async even though the in-memory
  store never awaits, so the SQL Server store won't change any call sites.
- **Atomic checkout.** `commit_sale` checks stock, decrements it and records
  the sale as one operation. In T-SQL that's one transaction
  (`BEGIN TRAN` ... `COMMIT`), decrementing with a conditional
  `UPDATE products SET stock = stock - @P1 WHERE id = @P2 AND stock >= @P1`
  and rolling back if any update affects 0 rows.
- **SQL-friendly types.** IDs are `i64` (`BIGINT IDENTITY`) and money is
  integer cents (`BIGINT`, not the T-SQL `MONEY` type). There are no floats.
  New IDs come back via `OUTPUT INSERTED.id`.
- **Price snapshots.** Line items copy name and unit price at sale time, so
  editing a product doesn't rewrite sales history.

Steps when that phase starts:

1. Uncomment the tiberius / bb8 / tokio-util dependencies in `Cargo.toml`
   and add `"net"` to tokio's features.
2. Add `migrations/` with T-SQL scripts for `products`, `sales` and
   `sale_items` tables. tiberius has no migration runner, so either run them
   with `sqlcmd` or add a tool such as `refinery` (which supports tiberius).
3. Add `src/store/mssql.rs` with `MssqlStore { pool: bb8::Pool<bb8_tiberius::ConnectionManager> }`
   implementing `PosStore`.
4. Add `PosError::Database(#[from] tiberius::error::Error)` plus a pool error variant.
5. Read `DATABASE_URL`, an ADO.NET-style connection string (copy
   `.env.example` to `.env`), parse it with `tiberius::Config::from_ado_string`,
   and build `MssqlStore` in `main.rs`.
6. Run the existing service tests against both stores. For local dev, the
   `mcr.microsoft.com/mssql/server` container image runs on Linux.
