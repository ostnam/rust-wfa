use lib::alignment_lib::{Alignment, Penalties};
use lib::validation_lib::*;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use clap::Parser;

/// Type used for CLI args parsing using clap.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ValidateArgs {
    #[clap(short, long)]
    parallel: bool,

    #[clap(long)]
    min_length: usize,

    #[clap(long)]
    max_length: usize,

    #[clap(long)]
    min_error: i32,

    #[clap(long)]
    max_error: i32,
}

fn run_validate_sma(args: ValidateArgs) {
    for cycle in 0..u32::MAX {
        let (computed_score, alignment, pens) = validate_sma(
            &AlignmentType::WavefrontNaive,
            args.min_length,
            args.max_length,
            args.min_error,
            args.max_error,
        );
        if computed_score == alignment.score {
            println!("Validation successful at cycle {}", cycle)
        } else {
            println!("Validation failed at cycle {}. The score of: {:?}, should be: {} for the penalties: {:?}.", cycle, alignment, computed_score, pens);
            panic!();
        }
    }
}

fn run_validate_sma_concurrent(args: ValidateArgs) {
    let num_threads = num_cpus::get();
    let (tx, rx): (
        Sender<(i32, Alignment, Penalties)>,
        Receiver<(i32, Alignment, Penalties)>,
    ) = mpsc::channel();
    let mut threads = Vec::new();

    for _ in 0..num_threads {
        let new_tx = tx.clone();
        threads.push(thread::spawn(move || {
            while new_tx
                .send(validate_sma(
                    &AlignmentType::WavefrontNaive,
                    args.min_length,
                    args.max_length,
                    args.min_error,
                    args.max_error,
                ))
                .is_ok()
            {}
        }));
    }

    for cycle in 0..=u64::MAX {
        match rx.recv() {
            Ok((computed_score, alignment, pens)) => {
                if computed_score == alignment.score {
                    println!("Validation successful at cycle {}", cycle);
                } else {
                    println!("Validation failed at cycle {}. The score of: {:?} should be {} for the penalties: {:?}.", cycle, alignment, computed_score, pens);
                    panic!();
                }
            }
            Err(_) => panic!(),
        }
    }
}

fn main() {
    let args = ValidateArgs::parse();

    if args.parallel {
        run_validate_sma_concurrent(args);
    } else {
        run_validate_sma(args);
    }
}
