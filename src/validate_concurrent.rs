use lib::validation_lib::*;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use num_cpus;

fn main() {
    let num_threads = num_cpus::get();
    
    let (tx, rx): (Sender<ValidationResult>, Receiver<ValidationResult>) = mpsc::channel();
    let mut threads = Vec::new();
    for i in 0..num_threads {
        let new_tx = tx.clone();
        threads.push(
            thread::spawn(move || {
                loop {
                    new_tx.send(
                        compare_alignment(&AlignmentType::WavefrontNaive,
                                            &AlignmentType::Reference,
                                            2,
                                            5,
                                            0,
                                            50,)
                    ).unwrap();
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
