use std::fs::File;
use std::io;
use std::io::Read;

use clap::Parser;
use csv::{Writer, WriterBuilder};
use qjsonrs::sync::Stream;

use crate::args::Args;
use crate::json_path::JsonIndexIterator;

mod args;
mod json_path;


fn main() {
    let args = Args::parse();
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init().expect("Could not setup logging!");

    let mut writer =WriterBuilder::new();
    writer.double_quote(false);

    let mut writer = writer.from_writer(io::stdout());
    for result in open_input(args.input) {
        let (key, value) = result.unwrap();
        // println!("{key} -> {value}");
        writer.write_record(&vec![
            format!("{}", key),
            format!("{}", value),
        ]).unwrap();
    }
}

fn open_input(input: Option<String>) -> JsonIndexIterator<Box<dyn Read>> {
    let mut stream: Box<dyn Read> = if let Some(input) = input {
        Box::new(File::open(input).unwrap())
    } else {
        Box::new(io::stdin())
    };
    let stream = Stream::from_read(stream).unwrap();
    let mut stream = JsonIndexIterator::new(stream);
    stream
}


#[cfg(test)]
mod tests {}