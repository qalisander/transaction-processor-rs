mod tr_processor;
mod data;

use anyhow::Result;
use csv::Trim;
use std::fmt::Error;
use std::io;




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
