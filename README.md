# tr_processor
Simple toy transaction processing engine, that reads a series of transactions from a CSV, updates client accounts.
Uses memory storage utilizing two hashmaps. Probably btreemap would be better for huger datasests. Also there is a point to consider persistent data structures like that https://github.com/orium/rpds
Rust decimal crate is used for better precision. Alas, 0.1 + 0.2 != 0.3_f64 is true in Rust.
With anyhow's divine assistane error handling became quite convineint.
Bugs are possible :)
