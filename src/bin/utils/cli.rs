use std::path::PathBuf;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// vscode debugger
    #[arg(long)]
    pub debugger: bool,

    /// path to file we're running
    pub path: Option<PathBuf>,
}
