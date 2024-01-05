use std::fs::File;
use std::io;
use std::io::Read;

use clap::Parser;
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

    let mut stream: Box<dyn Read> = if let Some(input) = args.input {
        Box::new(File::open(input).unwrap())
    } else {
        Box::new(io::stdin())
    };
    let stream = Stream::from_read(stream).unwrap();
    let mut stream = JsonIndexIterator::new(stream);


    for result in stream {
        let (key, value) = result.unwrap();
        println!("{key} -> {value}");
    }
}


#[cfg(test)]
mod tests {}