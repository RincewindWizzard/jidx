use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Path to config file
    #[arg(short, long)]
    config: Option<String>,

    /// no stdout printing
    #[arg(short, long)]
    pub(crate) quiet: bool,

}