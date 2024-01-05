use std::fmt::{Debug, Display};

use clap::Parser;
use qjsonrs::sync::TokenIterator;

use crate::args::Args;

mod args;
mod json_path;

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
    // read_json().unwrap();

    // parse_json_tokens(include_str!("../testdata/mars_weather.json"));
}

// fn parse_json_tokens(json_string: &str) -> Result<(), serde_json::Error> {
//     let mut deserializer = serde_json::Deserializer::from_str(json_string);
//
//     while let Some(token_result) = deserializer.next_token()? {
//         println!("{:?}", token_result);
//     }
//
//     Ok(())
// }


// fn read_json() -> anyhow::Result<()> {
//     let file = File::open(EXAMPLE_JSON)?;
//     let mut stream = Stream::from_read(file)?.index_paths();
//
//     loop {
//         match stream.next()? {
//             None => {
//                 break;
//             }
//             Some((key, value)) => {
//                 println!("{key} -> {value}");
//             }
//         }
//     }
//     Ok(())
// }


#[cfg(test)]
mod tests {

}