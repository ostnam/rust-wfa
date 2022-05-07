use lib::validation_lib::*;
use lib::alignment_lib::AlignmentAlgorithm;

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

    #[clap(short, long, default_value_t = u64::MAX)]
    /// Number of random pairings to validate.
    number: u64,
}

fn validate(args: ValidateArgs) -> bool {
    for cycle in 0..args.number {
        match compare_alignment(
            &AlignmentAlgorithm::Wavefront,
            &AlignmentAlgorithm::SWG,
            args.min_length,
            args.max_length,
            args.min_error,
            args.max_error,
        ) {
            ValidationResult::Passed => println!("Validation successful at cycle {}", cycle),
            ValidationResult::Failed(a) => {
                println!("Validation failed at cycle {}. \n {:?}", cycle, a);
                return false;
            }
        }
    }
    true
}

fn validate_concurrent(args: ValidateArgs) -> bool {
    let num_threads = num_cpus::get();
    let (tx, rx): (Sender<ValidationResult>, Receiver<ValidationResult>) = mpsc::channel();
    let mut threads = Vec::new();

    for _ in 0..num_threads {
        let new_tx = tx.clone();
        threads.push(thread::spawn(move || {
            while new_tx
                .send(compare_alignment(
                    &AlignmentAlgorithm::Wavefront,
                    &AlignmentAlgorithm::SWG,
                    args.min_length,
                    args.max_length,
                    args.min_error,
                    args.max_error,
                ))
                .is_ok()
            {}
        }));
    }

    for cycle in 1..=args.number {
        match rx.recv() {
            Ok(ValidationResult::Passed) => println!("Validation successful at cycle {}", cycle),
            Ok(ValidationResult::Failed(a)) => {
                println!("Validation failed at cycle {}. \n {:?}", cycle, a);
                return false;
            }
            Err(a) => {
                println!("{a}");
                return false;
            }
        }
    }
    true
}

fn main() {
    let args = ValidateArgs::parse();
    if args.parallel {
        validate_concurrent(args);
    } else {
        validate(args);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_250_parallel() {
        assert!(validate_concurrent(ValidateArgs {
            min_length: 0,
            max_length: 100,
            min_error: 0,
            max_error: 100,
            number: 250,
            parallel: true,
        }
        ));
    }
}
