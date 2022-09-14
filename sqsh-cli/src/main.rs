mod cli;
mod utils;
use clap::Parser;
use log::debug;
use sqsh::processors::{Adler32, Duplicate, CRC32};
use utils::{generate_file_stream, generate_stdout_stream};

fn main() -> std::io::Result<()> {
    let args = cli::Cli::parse();
    debug!("{args:?}");

    match args.command {
        cli::Commands::Duplicate { input, output } => {
            if let Some(path) = output {
                let mut stream = generate_file_stream::<Duplicate>(input, path)?;
                stream.consume()?;
            } else {
                let mut stream = generate_stdout_stream::<Duplicate>(input)?;
                stream.consume()?;
            };
        }
        cli::Commands::Adler32 { input } => {
            let mut stream = generate_stdout_stream::<Adler32>(input)?;
            stream.consume()?;
        }
        cli::Commands::CRC32 { input } => {
            let mut stream = generate_stdout_stream::<CRC32>(input)?;
            stream.consume()?;
        }
    };
    Ok(())
}
