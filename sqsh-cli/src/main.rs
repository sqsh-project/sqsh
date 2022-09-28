use clap::Parser;
use log::debug;
use sqsh::processors::{
    Adler32, Duplicate, RleClassicDecoder, RleClassicEncoder, TelemetryRleDecoder,
    TelemetryRleEncoder, CRC32,
};
use utils::generate_stdout_stream;
mod cli;
mod utils;

fn main() -> std::io::Result<()> {
    let args = cli::Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
    debug!("Configuration: {args:?}");

    let mut stream = match args.command {
        cli::Commands::Duplicate => {
            let processor = Duplicate::default();
            generate_stdout_stream(processor)
        }
        cli::Commands::Adler32 => {
            let processor = Adler32::new();
            generate_stdout_stream(processor)
        }
        cli::Commands::CRC32 => {
            let processor = CRC32::new();
            generate_stdout_stream(processor)
        }
        cli::Commands::Rle {
            threshold,
            decompress,
            mode,
        } => match (threshold, mode) {
            (Some(t), cli::RleMode::Classic) => {
                if decompress {
                    let processor = RleClassicDecoder::with_threshold(t);
                    generate_stdout_stream(processor)
                } else {
                    let processor = RleClassicEncoder::with_threshold(t);
                    generate_stdout_stream(processor)
                }
            }
            (Some(t), cli::RleMode::InfoByte) => {
                if decompress {
                    let processor = TelemetryRleDecoder::new();
                    generate_stdout_stream(processor)
                } else {
                    let processor = TelemetryRleEncoder::with_threshold(t as u8);
                    generate_stdout_stream(processor)
                }
            }
            (None, cli::RleMode::Classic) => {
                if decompress {
                    let processor = RleClassicDecoder::default();
                    generate_stdout_stream(processor)
                } else {
                    let processor = RleClassicEncoder::default();
                    generate_stdout_stream(processor)
                }
            }
            (None, cli::RleMode::InfoByte) => {
                if decompress {
                    let processor = TelemetryRleDecoder::default();
                    generate_stdout_stream(processor)
                } else {
                    let processor = TelemetryRleEncoder::default();
                    generate_stdout_stream(processor)
                }
            }
        },
    };
    stream.consume()?;
    Ok(())
}
