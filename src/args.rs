use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {

    /// Path to input file
    #[clap(index = 1)]
    pub(crate) input: Option<String>,

    /// verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    /// no stdout printing
    #[arg(short, long)]
    pub(crate) quiet: bool,

}