use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use std::{fs, io};

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, PartialEq)]
pub enum TypeOperation {
    deposit,
    withdrawal,
    dispute,
    resolve,
    chargeback,
}

#[derive(Debug, Deserialize)]
pub struct Operation {
    #[serde(rename = "type")]
    pub type_operation: TypeOperation,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

#[derive(Parser, Default, Debug)]
struct Args {
    // PathBuf must be used instead of String because there exist valid path characters
    // which are not valid String unicode
    input_file: PathBuf,
}

// Parse the CSV into a vector of Operation
pub fn parse() -> Result<Vec<Operation>> {
    let args = Args::parse();
    let file_reader = fs::File::open(args.input_file)?;

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true) // in case it is not a consistent file
        .trim(csv::Trim::All)
        .from_reader(io::BufReader::new(file_reader));

    let mut list_operations = vec![];
    for result in rdr.deserialize::<Operation>() {
        match result {
            Ok(v) => list_operations.push(v),
            Err(_) => continue, // if one line cannot be parsed, ignore it
        }
    }
    Ok(list_operations)
}
