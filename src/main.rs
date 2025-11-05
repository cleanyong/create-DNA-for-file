use clap::Parser;
use num_bigint::BigUint;
use num_traits::Zero;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

/// Size of the I/O buffer when reading from files/STDIN.
const BUFFER_SIZE: usize = 64 * 1024;

/// CLI arguments for the hash-to-bigint tool.
#[derive(Parser, Debug)]
#[command(
    name = "create-DNA-for-file",
    about = "Compute a SHA-256 hash for a file or STDIN and print it as a big integer"
)]
struct Args {
    /// Path to an input file; omit to read from STDIN.
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Emit the result in hexadecimal instead of decimal.
    #[arg(long = "hex")]
    hex: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse();
    let mut hasher = Sha256::new();

    if let Some(path) = args.file {
        hash_file(&path, &mut hasher)?;
    } else {
        hash_reader(&mut io::stdin().lock(), &mut hasher)?;
    }

    let digest = hasher.finalize();
    let bigint = BigUint::from_bytes_be(&digest);

    if args.hex {
        println!("{}", format_hex(&bigint));
    } else {
        println!("{}", bigint);
    }

    Ok(())
}

fn hash_file(path: &PathBuf, hasher: &mut Sha256) -> Result<(), String> {
    let mut file = File::open(path).map_err(|err| format!("failed to open file {path:?}: {err}"))?;
    hash_reader(&mut file, hasher)
}

fn hash_reader<R: Read>(reader: &mut R, hasher: &mut Sha256) -> Result<(), String> {
    let mut buffer = vec![0u8; BUFFER_SIZE];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buffer[..n]),
            Err(err) => return Err(format!("failed to read input: {err}")),
        }
    }
    Ok(())
}

fn format_hex(value: &BigUint) -> String {
    if value.is_zero() {
        return "0".to_string();
    }
    let mut hex = value.to_str_radix(16);
    if hex.len() % 2 != 0 {
        hex.insert(0, '0');
    }
    hex
}
