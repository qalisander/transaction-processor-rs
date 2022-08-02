# tr_processor
Simple toy transaction processing engine, that reads a series of transactions from a CSV, updates client accounts.

Uses memory storage utilizing two hashmaps. Probably btreemap would be better for huger datasets. Also, there is a point to consider persistent data structures like that https://github.com/orium/rpds

Rust decimal crate is used for better precision. Alas, 0.1 + 0.2 != 0.3_f64 is true in Rust.

With anyhow's divine assistance error handling became quite convenient.

Bugs happens :)

Assumptions
- In case of disputing withdrawal transactions, held funds amount doesn't increase. As like available amount doesn't change. So client will not see any difference until dispute of withdrawal transaction will be chargebacked.
- Depositing or withdrawing negative amount of money will be ignored with a corresponding error.
