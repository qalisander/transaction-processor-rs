use std::env::args;
use std::fs::File;
use std::io;

use anyhow::{Result};
use csv::Trim;

use crate::data::*;
use crate::tr_processor::TrProcessor;

mod data;
mod tr_processor;

// TODO: pay attention to f64, use decimal https://docs.rs/rust_decimal/latest/rust_decimal/
// TODO: use another writer io::stderr() for errors

fn main() {
    let string = args().nth(1).expect("No file argument!");
    let file = File::open(string).unwrap();
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .comment(Some(b'#'))
        .from_reader(file);

    let iter = reader
        .deserialize()
        .into_iter()
        .map::<Result<Tr>, _>(|record| {
            let record: TrRecord = record?;
            let tr: Tr = record.try_into()?;
            Ok(tr)
        })
        .inspect(|res| {
            if let Err(err) = res {
                eprintln!("{}", err)
            }
        })
        .flatten();
    
    let mut processor = TrProcessor::new();
    processor.process(iter);
    let mut writer = csv::Writer::from_writer(io::stdout());
    for info in processor.get_client_records() {
        writer.serialize(info).unwrap_or_else(|err|{
            eprintln!("{}", err)
        });
    }
}

