mod cli;
use clap::Parser;
use sqsh::processors::Duplicate;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

fn main() -> std::io::Result<()> {
    let args = cli::Cli::parse();
    println!("{args:?}");

    let mut stream = match args.command {
        cli::Commands::Duplicate { input, output } => {
            let opath = match output {
                Some(path) => path,
                None => {
                    let mut tmp = input.clone();
                    tmp.set_extension("raw");
                    tmp
                }
            };
            let i = File::open(input)?;
            let o = File::create(opath)?;
            let bufreader = BufReader::new(i);
            let writer = BufWriter::new(o);
            let processor = Duplicate::new();
            sqsh::core::Stream::new(bufreader, writer, processor)
        }
    };
    stream.consume()?;
    Ok(())
}
