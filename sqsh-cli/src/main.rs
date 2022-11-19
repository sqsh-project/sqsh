use clap::Parser;
use log::debug;
use sqsh::processors::{
    Adler32, ConditionalRleDecoder, ConditionalRleEncoder, Duplicate, LossyRleDecoder,
    LossyRleEncoder, RleClassicDecoder, RleClassicEncoder, TelemetryRleDecoder,
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
            repetitions,
            threshold,
            decompress,
            mode,
            order,
            bits,
        } => match mode {
            cli::RleMode::Conditional => {
                if decompress {
                    let processor = ConditionalRleDecoder::with_order_with_bitlength(order, bits);
                    generate_stdout_stream(processor)
                } else {
                    let processor = ConditionalRleEncoder::with_order_with_bitlength(order, bits);
                    generate_stdout_stream(processor)
                }
            }
            cli::RleMode::Classic => {
                if decompress {
                    let processor = RleClassicDecoder::with_threshold(repetitions);
                    generate_stdout_stream(processor)
                } else {
                    let processor = RleClassicEncoder::with_threshold(repetitions);
                    generate_stdout_stream(processor)
                }
            }
            cli::RleMode::Infobyte => {
                if decompress {
                    let processor = TelemetryRleDecoder::default();
                    generate_stdout_stream(processor)
                } else {
                    let processor = TelemetryRleEncoder::with_threshold(threshold);
                    generate_stdout_stream(processor)
                }
            }
            cli::RleMode::Lossy => {
                if decompress {
                    let processor = LossyRleDecoder::default();
                    generate_stdout_stream(processor)
                } else {
                    let processor = LossyRleEncoder::with_threshold(repetitions);
                    generate_stdout_stream(processor)
                }
            }
        },
    };
    stream.consume()?;
    Ok(())
}
