use std::fmt::Display;

use clap::{Parser, Subcommand};

/// Command-line Interface (CLI) for the sqsh library
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Subcommand to be executed
    #[clap(subcommand)]
    pub command: Commands,

    /// Control verbose output (e.g. -vv for info level)
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

/// Commands to be executed by the CLI
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Duplicate the input to the output
    Duplicate,
    /// Calculate Adler32 checksum
    Adler32,
    /// Calculate CRC32 checksum
    CRC32,
    /// En:Decode input using RLE (two modes)
    Rle {
        /// Max allowed repetition which are not compressed
        #[clap(short, long, value_parser)]
        threshold: Option<usize>,

        #[clap(long, value_parser, default_value_t = RleMode::Classic)]
        mode: RleMode,

        /// Decompress input
        #[clap(short, long, value_parser, default_value_t = false)]
        decompress: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum RleMode {
    #[clap(alias = "info", alias = "Info", alias = "i")]
    InfoByte,
    #[clap(alias = "classic", alias = "Classic", alias = "c")]
    Classic,
}

impl Display for RleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InfoByte => write!(f, "InfoByte"),
            Self::Classic => write!(f, "Classic"),
        }
    }
}
