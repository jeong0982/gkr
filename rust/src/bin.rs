use clap::{Parser, Subcommand};
use r1cs_file::*;
use std::{fs::File, io::Result};
use wtns_file::*;

extern crate gkr;
use gkr::aggregator::prove_all;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Prove {
        #[arg(short, long)]
        circuit: String,
        #[arg(short, long, num_args=0..)]
        inputs: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Prove { circuit, inputs }) => {
            let circuit_path = circuit.clone();
            let input_paths = inputs.clone();
            prove_all(circuit_path, input_paths);
        },
        None => {},
    }

    Ok(())
}
