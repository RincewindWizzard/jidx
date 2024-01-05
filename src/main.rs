use std::fmt::{Debug, Display};
use std::fs::File;

use clap::Parser;
use log::debug;
use qjsonrs::sync::{Stream, TokenIterator};

use crate::args::Args;
use crate::json_path::{JsonPath};
use crate::json_path::ToJsonPath;
use crate::token::{JsonIndexable, StackElement};
use crate::token::StackElement::ArrayIndex;

mod json_path;
mod token;
mod args;

const EXAMPLE_JSON: &str = "./testdata/mars_weather.json";
const BUFFER_SIZE: usize = 1024;


fn main() {
    let args = Args::parse();
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init().expect("Could not setup logging!");
    read_json().unwrap();
}

fn read_json() -> anyhow::Result<()> {
    let file = File::open(EXAMPLE_JSON)?;
    let mut stream = Stream::from_read(file)?.index_paths();

    loop {
        match stream.next()? {
            None => {
                break;
            }
            Some((key, value)) => {
                println!("{key} -> {value}");
            }
        }
    }
    Ok(())
}
