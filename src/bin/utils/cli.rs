use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Start in debug mode for lexing
    #[arg(short, long)]
    pub debug: bool,

    #[arg(long)]
    pub debugger: bool,
}
