use sqsh::core::{Consume, Process};
use std::{
    io::{BufReader, BufWriter},
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
pub(crate) fn generate_stdout_stream(processor: impl Process + 'static) -> Box<dyn Consume> {
    let output = std::io::stdout();
    let input = std::io::stdin();
    let bufreader = BufReader::new(input);
    let writer = BufWriter::new(output);
    let stream = sqsh::core::Stream::new(bufreader, writer, processor);
    Box::new(stream)
}
