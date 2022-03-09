use clap::Parser;
use lib::validation_lib::*;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use num_cpus;

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

fn validate(args: ValidateArgs) {
    for cycle in 0..u32::MAX {
        match compare_alignment(&AlignmentType::WavefrontNaive,
                                    &AlignmentType::Reference,
                                    args.min_length,
                                    args.max_length,
                                    args.min_error,
                                    args.max_error) {
            ValidationResult::Passed => println!("Validation successful at cycle {}", cycle),
            ValidationResult::Failed(a) => {
                println!("Validation failed at cycle {}. \n {:?}", cycle, a);
                return ();
            }
        }
    }
}


fn validate_concurrent(args: ValidateArgs) {
    let num_threads = num_cpus::get();
    let (tx, rx): (Sender<ValidationResult>, Receiver<ValidationResult>) = mpsc::channel();
    let mut threads = Vec::new();

    for i in 0..num_threads {
        let new_tx = tx.clone();
        threads.push(
            thread::spawn(move || {
                loop {
                    match new_tx.send(
                        compare_alignment(&AlignmentType::WavefrontNaive,
                                          &AlignmentType::Reference,
                                          args.min_length,
                                          args.max_length,
                                          args.min_error,
                                          args.max_error)
                        ) {
                        Ok(_) => (),
                        Err(_) => break,
                    }
                }
            }
            )
        )
        }

    for cycle in 0..=u64::MAX {
        match rx.recv() {
            Ok(ValidationResult::Passed) => println!("Validation successful at cycle {}", cycle),
            Ok(ValidationResult::Failed(a)) => {
                println!("Validation failed at cycle {}. \n {:?}", cycle, a);
                return ();
            },
            Err(_) => panic!(),
        }
    }
}

fn main() {
    let args = ValidateArgs::parse();

    if args.parallel {
        validate_concurrent(args);
    } else {
        validate(args);
    }
}
