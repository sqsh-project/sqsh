use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
// #[clap(name = "td")]
// #[clap(bin_name = "td")]
pub struct Cli {
    /// Mean of Normal Distribution
    #[clap(short, long, value_parser)]
    pub mean: f64,

    /// Standard Deviation of Normal Distribution
    #[clap(short, long = "std", value_parser)]
    pub std_dev: f64,

    /// Number of values to be generated
    #[clap(short, long, value_parser)]
    pub num: usize,

    /// Print generated data to STDOUT
    #[clap(short, long, value_parser, default_value_t = false)]
    pub print: bool,

    /// Seed value for PRNG
    #[clap(long, value_parser)]
    pub seed: Option<u64>,

    /// Endianess of output
    #[clap(short, value_enum, long, default_value_t = Endianess::Native)]
    pub endianess: Endianess,

    /// Datatype of output
    #[clap(short, value_enum, long, default_value_t = Datatype::Float)]
    pub datatype: Datatype,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Endianess {
    #[clap(alias = "be", alias = "b")]
    Big,
    #[clap(alias = "le", alias = "l")]
    Little,
    #[clap(alias = "ne", alias = "n")]
    Native,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Datatype {
    #[clap(alias = "f32", alias = "f")]
    Float,
    #[clap(alias = "f64", alias = "d")]
    Double,
}
