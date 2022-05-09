use lib::alignment_lib::*;

use std::sync::mpsc::{self, Receiver, Sender}; // Parallel validation.
use std::{fmt, thread}; // Parallel validation and error messages.

use rand::{thread_rng, Rng}; // Validation case generation.

use clap::Parser;

fn main() {
    let args = ValidateArgs::parse();
    if args.parallel {
        validate_concurrent(args);
    } else {
        validate(args);
    }
}

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
        match run_validation(
            args.min_length,
            args.max_length,
            args.min_error,
            args.max_error,
        ) {
            Ok(_) => println!("Validation successful at cycle {}", cycle),
            Err(a) => {
                println!("Validation failed at cycle {}. \n {:?}", cycle, a);
                return false;
            }
        }
    }
    true
}

fn validate_concurrent(args: ValidateArgs) -> bool {
    let num_threads = num_cpus::get();
    let (tx, rx): (
        Sender<Result<(), ValidationError>>,
        Receiver<Result<(), ValidationError>>,
    ) = mpsc::channel();
    let mut threads = Vec::new();

    for _ in 0..num_threads {
        let new_tx = tx.clone();
        threads.push(thread::spawn(move || {
            while new_tx
                .send(run_validation(
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
            Ok(Ok(_)) => println!("Validation successful at cycle {}", cycle),
            Ok(Err(a)) => {
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

mod validation_generation {
    use rand::distributions::{Alphanumeric, Distribution, Standard};
    use rand::{thread_rng, Rng};

    enum MutationType {
        Insertion,
        Deletion,
        Substitution,
    }

    // Allows to randomly generate a MutationType.
    impl Distribution<MutationType> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MutationType {
            match rng.gen_range(0..=2) {
                0 => MutationType::Insertion,
                1 => MutationType::Deletion,
                _ => MutationType::Substitution,
            }
        }
    }

    pub fn random_string(min_length: usize, max_length: usize) -> String {
        let mut rng = thread_rng();
        let length = rng.gen_range(min_length..max_length);

        (&mut rng)
            .sample_iter(Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }

    fn gen_new_char() -> char {
        let mut rng = thread_rng();
        (&mut rng)
            .sample_iter(Alphanumeric)
            .take(1)
            .map(char::from)
            .collect::<Vec<char>>()[0]
    }

    fn gen_new_char_different(a: char) -> char {
        loop {
            let c = gen_new_char();
            if c != a {
                return c;
            }
        }
    }

    pub fn mutate(text: &str, min_error: i32, max_error: i32) -> String {
        let mut rng = thread_rng();
        let mut mutated: Vec<char> = text.chars().collect();
        let error_rate: i32 = rng.gen_range(min_error..max_error);
        let final_err_count: i32 = (error_rate * (mutated.len() as i32)) / 100;

        for _ in 0..final_err_count {
            let position: usize = rng.gen_range(0..mutated.len());
            let mutation: MutationType = rand::random();
            if let MutationType::Insertion = mutation {
                mutated.insert(position, gen_new_char());
            }
            if let MutationType::Deletion = mutation {
                mutated.remove(position);
            }
            if let MutationType::Substitution = mutation {
                mutated[position] = gen_new_char_different(mutated[position]);
            }
        }
        mutated.into_iter().collect()
    }
}

fn check_score_error(alignment: Alignment, pens: &Penalties) -> Option<IncorrectScore> {
    let computed_score = compute_score_from_alignment(&alignment, pens);
    if alignment.score == computed_score {
        None
    } else {
        Some(IncorrectScore {
            alignment,
            computed_score,
        })
    }
}

fn compute_score_from_alignment(alignment: &Alignment, pens: &Penalties) -> i32 {
    let mut computed_score: i32 = 0;
    let mut current_layer: AlignmentLayer = AlignmentLayer::Matches;
    for (c1, c2) in alignment
        .query_aligned
        .chars()
        .zip(alignment.text_aligned.chars())
    {
        if c1 == '-' {
            computed_score += pens.extd_pen
                + match current_layer {
                    AlignmentLayer::Deletes => 0,
                    _ => pens.open_pen,
                };
            current_layer = AlignmentLayer::Deletes;
        } else if c2 == '-' {
            computed_score += pens.extd_pen
                + match current_layer {
                    AlignmentLayer::Inserts => 0,
                    _ => pens.open_pen,
                };
            current_layer = AlignmentLayer::Inserts;
        } else {
            current_layer = AlignmentLayer::Matches;
            if c1 != c2 {
                computed_score += pens.mismatch_pen;
            }
        }
    }
    computed_score
}

struct IncorrectScore {
    alignment: Alignment,
    computed_score: i32,
}

impl fmt::Debug for IncorrectScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The score of {:?} is incorrect. It should be {}, after recalculating it using the number of mismatches/gap in the alignment", self.alignment, self.computed_score)
    }
}

/// This type returns every type of error we can get in a validation case.
#[derive(Debug)]
enum ValidationError {
    /// This variant is for the case where the score is incorrect: it doesn't match the
    /// alignment.
    IncorrectScore(IncorrectScore),

    /// This variant is for the case where both alignments have different scores. There can be only
    /// one optimal alignment score, so at least one of them is wrong.
    ScoresDiffer(ScoresDiffer),

    /// For the case when one alignment failed (returned an AlignmentError) but not the other.
    AlignmentFailure((AlignmentError, AlignmentAlgorithm)),
}

struct ScoresDiffer {
    query: String,
    text: String,
    a_score: i32,
    b_score: i32,
    query_aligned_a: String,
    text_aligned_a: String,
    query_aligned_b: String,
    text_aligned_b: String,
    pens: Penalties,
}

impl fmt::Debug for ScoresDiffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error comparing the alignment of {} with {}. The first method finds a score of {} while the second gives {}. \n First alignment: {}\n{}\nSecond alignment:{}\n{}\nPenalties:{:?}", self.query, self.text, self.a_score, self.b_score, self.query_aligned_a, self.text_aligned_a, self.query_aligned_b, self.text_aligned_b, self.pens)
    }
}

/// This function generates a case, run the alignment, and then checks that it is valid.
fn run_validation(
    min_length: usize,
    max_length: usize,
    min_error: i32,
    max_error: i32,
) -> Result<(), ValidationError> {
    // generate 2 strings
    let mut text = validation_generation::random_string(min_length, max_length);
    let mut query = validation_generation::mutate(&text, min_error, max_error);
    if query.len() > text.len() {
        std::mem::swap(&mut query, &mut text);
    }

    // generate pens
    let mut rng = thread_rng();
    let pens = Penalties {
        mismatch_pen: rng.gen_range(1..100),
        open_pen: rng.gen_range(1..100),
        extd_pen: rng.gen_range(1..100),
    };

    // align them using the method
    let a_result = lib::wavefront_alignment::wavefront_align(&query, &text, &pens);
    let b_result = lib::reference::affine_gap_align(&query, &text, &pens);

    match (a_result, b_result) {
        (Ok(a), Ok(b)) if a.score == b.score => {
            // Both functions aligned succesfully with the same score.
            match (check_score_error(a, &pens), check_score_error(b, &pens)) {
                (None, None) => Ok(()),
                (Some(a), _) => Err(ValidationError::IncorrectScore(a)),
                (_, Some(a)) => Err(ValidationError::IncorrectScore(a)),
            }
        }
        (Ok(a), Ok(b)) => Err(ValidationError::ScoresDiffer(ScoresDiffer {
            query,
            text,
            a_score: a.score,
            b_score: b.score,
            query_aligned_a: a.query_aligned,
            text_aligned_a: a.text_aligned,
            query_aligned_b: b.query_aligned,
            text_aligned_b: b.text_aligned,
            pens,
        })),

        (Err(_), Err(_)) => Ok(()), // both alignment functions didn't work, let's assume it's normal.
        (Err(a), Ok(_)) => Err(ValidationError::AlignmentFailure((
            a,
            AlignmentAlgorithm::Wavefront,
        ))),
        (Ok(_), Err(a)) => Err(ValidationError::AlignmentFailure((
            a,
            AlignmentAlgorithm::SWG,
        ))),
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
        }));
    }
}
