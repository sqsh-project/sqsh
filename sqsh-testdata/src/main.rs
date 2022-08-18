use std::io::{stdout, Write};

use byteorder::{BigEndian, LittleEndian, NativeEndian, WriteBytesExt};
use clap::Parser;
use rand::{thread_rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal, NormalError};

mod cli;

fn main() -> Result<(), NormalError> {
    let args = cli::Cli::parse();
    let mut output = stdout();

    match args.datatype {
        cli::Datatype::Double => {
            let values = get_normal_distribution_f64(args.mean, args.std_dev, args.num, args.seed)?;
            if args.print {
                println!("{:?}", values)
            } else {
                write_to_disk_f64(&values, &mut output, args.endianess).unwrap();
            }
        }
        cli::Datatype::Float => {
            let values = get_normal_distribution_f32(
                args.mean as f32,
                args.std_dev as f32,
                args.num,
                args.seed,
            )?;
            if args.print {
                println!("{:?}", values)
            } else {
                write_to_disk_f32(&values, &mut output, args.endianess).unwrap();
            }
        }
    }
    Ok(())
}

fn get_normal_distribution_f64(
    mean: f64,
    std_dev: f64,
    size: usize,
    seed: Option<u64>,
) -> Result<Vec<f64>, NormalError> {
    let normal = Normal::new(mean, std_dev)?;
    let values: Vec<f64> = match seed {
        Some(s) => {
            let mut rng = ChaCha8Rng::seed_from_u64(s);
            std::iter::repeat(0f64)
                .take(size)
                .map(|_| normal.sample(&mut rng))
                .collect()
        }
        None => {
            let mut rng = thread_rng();
            std::iter::repeat(0f64)
                .take(size)
                .map(|_| normal.sample(&mut rng))
                .collect()
        }
    };

    Ok(values)
}

fn to_u8_f64(data: &[f64], endianess: cli::Endianess) -> Vec<u8> {
    let mut wtr = Vec::new();
    match endianess {
        cli::Endianess::Big => {
            for val in data.iter() {
                wtr.write_f64::<BigEndian>(*val).unwrap();
            }
            wtr
        }
        cli::Endianess::Little => {
            for val in data.iter() {
                wtr.write_f64::<LittleEndian>(*val).unwrap();
            }
            wtr
        }
        cli::Endianess::Native => {
            for val in data.iter() {
                wtr.write_f64::<NativeEndian>(*val).unwrap();
            }
            wtr
        }
    }
}

fn write_to_disk_f64<W: Write>(
    values: &[f64],
    path: &mut W,
    endianess: cli::Endianess,
) -> std::io::Result<()> {
    let data = to_u8_f64(values, endianess);
    path.write_all(&data)?;
    Ok(())
}

fn get_normal_distribution_f32(
    mean: f32,
    std_dev: f32,
    size: usize,
    seed: Option<u64>,
) -> Result<Vec<f32>, NormalError> {
    let normal = Normal::new(mean, std_dev)?;
    let values: Vec<f32> = match seed {
        Some(s) => {
            let mut rng = ChaCha8Rng::seed_from_u64(s);
            std::iter::repeat(0f32)
                .take(size)
                .map(|_| normal.sample(&mut rng))
                .collect()
        }
        None => {
            let mut rng = thread_rng();
            std::iter::repeat(0f32)
                .take(size)
                .map(|_| normal.sample(&mut rng))
                .collect()
        }
    };

    Ok(values)
}

fn to_u8_f32(data: &[f32], endianess: cli::Endianess) -> Vec<u8> {
    let mut wtr = Vec::new();
    match endianess {
        cli::Endianess::Big => {
            for val in data.iter() {
                wtr.write_f32::<BigEndian>(*val).unwrap();
            }
            wtr
        }
        cli::Endianess::Little => {
            for val in data.iter() {
                wtr.write_f32::<LittleEndian>(*val).unwrap();
            }
            wtr
        }
        cli::Endianess::Native => {
            for val in data.iter() {
                wtr.write_f32::<NativeEndian>(*val).unwrap();
            }
            wtr
        }
    }
}

fn write_to_disk_f32<W: Write>(
    values: &[f32],
    path: &mut W,
    endianess: cli::Endianess,
) -> std::io::Result<()> {
    let data = to_u8_f32(values, endianess);
    path.write_all(&data)?;
    Ok(())
}
