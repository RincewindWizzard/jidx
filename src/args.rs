use std::str::FromStr;

use clap::Parser;

use crate::args::OutputFormat::CSV;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Path to input file
    /// If omitted stdin is used
    #[clap(index = 1)]
    pub(crate) input: Option<String>,

    /// verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    /// no stdout printing
    #[arg(short, long)]
    pub(crate) quiet: bool,

    /// The output format for stdout
    #[arg(short, long, default_value = "CSV")]
    pub(crate) output: OutputFormat,

}

#[derive(Parser, Clone, Debug)]
pub(crate) enum OutputFormat {
    CSV,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        if s == "csv" {
            return Ok(CSV);
        } else {
            Err(format!("Could not find output format!"))
        }
    }
}