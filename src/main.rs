mod tr_processor;

use anyhow::Result;
use csv::Trim;
use serde::{Deserialize, Serialize};
use std::fmt::Error;
use std::io;

//usage of #[serde(tag = "type")] is not possible because of bug
//https://github.com/BurntSushi/rust-csv/issues/278
#[derive(Deserialize)]
struct TrCsvRow {
    #[serde(alias = "type")]
    tr_type: String,
    client: u16,
    tx: u32,
    amount: Option<f64>,
}


#[derive(Debug, Copy, Clone)]
enum Tr {
    Deposit { client: u16, tx: u32,  amount: f64 },
    Withdrawal { client: u16, tx: u32, amount: f64 },
    Dispute { client: u16, tx: u32 },
    Resolve { client: u16, tx: u32 },
    Chargeback { client: u16, tx: u32 },
}

impl TryFrom<TrCsvRow> for Tr {
    type Error = &'static str;

    fn try_from(csv_row: TrCsvRow) -> std::result::Result<Self, Self::Error> {
        let client = csv_row.client;
        let tx = csv_row.tx;

        match &*csv_row.tr_type {
            "deposit" => match csv_row.amount {
                None => Err("Not valid amount!"),
                Some(amount) => Ok(Tr::Deposit { client, tx, amount }),
            },
            "withdrawal" => match csv_row.amount {
                None => Err("Not valid amount!"),
                Some(amount) => Ok(Tr::Withdrawal { client, tx, amount }),
            },
            "dispute" => Ok(Tr::Dispute {tx, client}),
            "resolve" => Ok(Tr::Resolve {tx, client}),
            "chargeback" => Ok(Tr::Chargeback {tx, client}),
            _ => Err("Not valid csv row!"),
        }
    }
}
// TODO: abstract over iterator of transaction
// TODO: pay attention to f64, use decimal
// https://docs.rs/rust_decimal/latest/rust_decimal/

fn main() -> Result<()> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .comment(Some(b'#'))
        .from_reader(io::stdin());
    for result in reader.deserialize() {
        let tr_csv_row: TrCsvRow = result?;
        let tr: Tr = tr_csv_row.try_into().unwrap();
        println!("{:?}", tr);
    }

    Ok(())
}
