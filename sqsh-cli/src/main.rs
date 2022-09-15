use clap::Parser;
use log::debug;
use sqsh::processors::{Adler32, Duplicate, RleClassicDecoder, RleClassicEncoder, CRC32};
use utils::generate_stdout_stream;
mod cli;
mod utils;

fn main() -> std::io::Result<()> {
    let args = cli::Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
    debug!("Configuration: {args:?}");

    match args.command {
        cli::Commands::Duplicate { input } => {
            let d = Duplicate::default();
            let mut stream = generate_stdout_stream(input, d)?;
            stream.consume()?;
        }
        cli::Commands::Adler32 { input } => {
            let processor = Adler32::new();
            let mut stream = generate_stdout_stream(input, processor)?;
            stream.consume()?;
        }
        cli::Commands::CRC32 { input } => {
            let processor = CRC32::new();
            let mut stream = generate_stdout_stream(input, processor)?;
            stream.consume()?;
        }
        cli::Commands::ClassicRLE {
            input,
            threshold,
            decompress,
        } => match threshold {
            Some(t) => {
                if decompress {
                    let processor = RleClassicDecoder::with_threshold(t);
                    let mut stream = generate_stdout_stream(input, processor)?;
                    stream.consume()?;
                } else {
                    let processor = RleClassicEncoder::with_threshold(t);
                    let mut stream = generate_stdout_stream(input, processor)?;
                    stream.consume()?;
                }
            }
            None => {
                if decompress {
                    let processor = RleClassicDecoder::default();
                    let mut stream = generate_stdout_stream(input, processor)?;
                    stream.consume()?;
                } else {
                    let processor = RleClassicEncoder::default();
                    let mut stream = generate_stdout_stream(input, processor)?;
                    stream.consume()?;
                }
            }
        },
    };
    Ok(())
}
