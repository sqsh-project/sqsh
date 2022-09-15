use sqsh::core::{Process, Stream};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Stdout},
    path::PathBuf,
};

/// Boilerplate for generating a stream from a file to a file
pub(crate) fn generate_file_stream<P: Process + Default>(
    input: PathBuf,
    output: PathBuf,
) -> std::io::Result<Stream<BufReader<File>, BufWriter<File>, P>> {
    let i = File::open(input)?;
    let o = File::create(output)?;
    let bufreader = BufReader::new(i);
    let writer = BufWriter::new(o);
    let processor = Default::default();
    let stream = sqsh::core::Stream::new(bufreader, writer, processor);
    Ok(stream)
}

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
) -> std::io::Result<Stream<BufReader<File>, BufWriter<Stdout>, P>> {
    let output = std::io::stdout();
    let i = File::open(input)?;
    let bufreader = BufReader::new(i);
    let writer = BufWriter::new(output);
    let processor = Default::default();
    let stream = sqsh::core::Stream::new(bufreader, writer, processor);
    Ok(stream)
}
