use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Command-line Interface (CLI) for the sqsh library
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Subcommand to be executed
    #[clap(subcommand)]
    pub command: Commands,
}

/// Commands to be executed by the CLI
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Duplicate the input to the output
    Duplicate {
        /// Input file
        #[clap(value_parser)]
        input: PathBuf,

        /// Output file
        #[clap(value_parser)]
        output: Option<PathBuf>,
    },
}
