mod cli;
mod utils;
use clap::Parser;
use sqsh::processors::{Adler32, Duplicate, CRC32};
use utils::{generate_file_stream, generate_output_filename, generate_stdout_stream};

fn main() -> std::io::Result<()> {
    let args = cli::Cli::parse();
    println!("{args:?}");

    match args.command {
        cli::Commands::Duplicate { input, output } => {
            let output = match output {
                Some(path) => path,
                None => generate_output_filename(input.clone()),
            };
            let mut stream = generate_file_stream::<Duplicate>(input, output)?;
            stream.consume()?;
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
