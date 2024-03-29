# Transaction Processor
Simple toy transaction processing engine, that reads a series of transactions from a CSV, updates client accounts.

The transaction processor uses memory storage with two hashmaps. Also, there is a point to consider persistent data structures (https://github.com/orium/rpds), in case of tremendous amount of transactions. Especially when few disputes occurs, there is no point to store all transactions in memory during program execution.

Rust decimal crate is used for better precision. Alas, 0.1 + 0.2 != 0.3_f64 is true in Rust.

With anyhow's divine assistance error handling became quite convenient.

Bugs happen :)

Assumptions
- In case of disputing withdrawal transactions, held funds amount doesn't increase. As like available amount doesn't change. So client will not see any difference until dispute of withdrawal transaction will be chargebacked.
- Depositing or withdrawing negative amount or zero of money will be ignored with a corresponding error.
- If client (id) specified by the dispute/resolved/chargeback doesn't math with existing tr (id). This transaction will be ignored, and logged in error output.
- Amount of client's funds can become negative in case deposit transaction been disputed (amount of withdrawal grater than funds available). Any withdrawal won't happen because of negative amount. When chargeback occurs, client will be blocked anyway. Then debt collectors will do their job :)

### Changelog 
0.3.0 Fixing transaction shadowing

0.2.0 Fixing invalid behaviour with negative amount in deposit and withdrawal transactions.

0.1.0 Initial version of toy transaction processing engine.