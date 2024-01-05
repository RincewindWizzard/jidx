
use clap::Parser;


use crate::args::Args;

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
}



#[cfg(test)]
mod tests {

}