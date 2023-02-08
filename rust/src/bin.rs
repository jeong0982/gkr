use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::{io::Result, process::Command};

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
    MockGroth {
        #[arg(short, long)]
        zkey: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Prove { circuit, inputs }) => {
            let circuit_path = circuit.clone();
            let input_paths = inputs.clone();
            prove_all(circuit_path, input_paths);
        }
        Some(Commands::MockGroth { zkey }) => {
            println!("mock groth16 running..");
            let output = Command::new("snarkjs")
                .arg("zkey")
                .arg("verify")
                .arg("aggregated.r1cs")
                .arg("pot.ptau")
                .arg(zkey.clone())
                .output()
                .expect("zkey verification failed");
            std::io::stdout().write_all(&output.stdout).unwrap();
            let output = Command::new("snarkjs")
                .arg("groth16")
                .arg("prove")
                .arg(zkey.clone())
                .arg("witness.wtns")
                .arg("proof.json")
                .arg("public.json")
                .output()
                .expect("proving failed");
            std::io::stdout().write_all(&output.stdout).unwrap();
            println!("Aggregation is done.");
        }
        None => {}
    }

    Ok(())
}
