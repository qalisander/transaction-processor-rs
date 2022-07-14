mod tr_processor;
mod data;
use anyhow::Result;
use csv::Trim;
use std::io;
use crate::data::*;
use crate::tr_processor::TrProcessor;


// TODO: pay attention to f64, use decimal https://docs.rs/rust_decimal/latest/rust_decimal/
// TODO: use another writer io::stderr() for errors

fn main() -> Result<()> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .comment(Some(b'#'))
        .from_reader(io::stdin());
    for result in reader.deserialize() {
        let tr_csv_row: TrRecord = result?;
        let tr: Tr = tr_csv_row.try_into().unwrap();
        println!("{:?}", tr);
    }

    let processor = TrProcessor::new();
    
    let mut writer = csv::Writer::from_writer(io::stdout());
    for info in processor.get_client_records() {
        writer.serialize(info)?;
    }

    Ok(())
}
