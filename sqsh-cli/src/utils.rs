use sqsh::core::{Process, Stream};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Stdout},
    path::PathBuf,
};

#[allow(dead_code)]
/// Function to automatically change the file extension
pub(crate) fn generate_output_filename(input: PathBuf) -> PathBuf {
    let mut tmp = input;
    tmp.set_extension("raw");
    tmp
}

/// Boilerplate for generating a stream from a file to stdout
pub(crate) fn generate_stdout_stream<P: Process + Default>(
    input: PathBuf,
    processor: P,
) -> std::io::Result<Stream<BufReader<File>, BufWriter<Stdout>, P>> {
    let output = std::io::stdout();
    let i = File::open(input)?;
    let bufreader = BufReader::new(i);
    let writer = BufWriter::new(output);
    let stream = sqsh::core::Stream::new(bufreader, writer, processor);
    Ok(stream)
}
