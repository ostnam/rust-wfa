use crate::reference::affine_gap_align;
use crate::wavefront_alignment::wavefront_align;
use crate::{alignment_lib::*, reference};
use core::fmt;
use rand::distributions::{Alphanumeric, Distribution, Standard};
use rand::{thread_rng, Rng};
use std::fmt::Debug;

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

fn random_string(min_length: usize, max_length: usize) -> String {
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

fn mutate(text: &str, min_error: i32, max_error: i32) -> String {
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

pub enum ValidationResult {
    Passed,
    Failed(ValidationFailureType),
}

#[derive(Debug)]
pub enum ValidationFailureType {
    ScoreMismatch(ScoreMismatch),
    AlignResultMismatch(AlignResultMismatch),
}

pub struct ScoreMismatch {
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

impl Debug for ScoreMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error comparing the alignment of {} with {}. The first method finds a score of {} while the second gives {}. \n First alignment: {}\n{}\nSecond alignment:{}\n{}\nPenalties:{:?}", self.query, self.text, self.a_score, self.b_score, self.query_aligned_a, self.text_aligned_a, self.query_aligned_b, self.text_aligned_b, self.pens)
    }
}

#[derive(Debug)]
pub struct AlignResultMismatch {
    failed_type: AlignmentAlgorithm,
    failed: AlignError,
}

pub fn compare_alignment(
    a_type: &AlignmentAlgorithm,
    b_type: &AlignmentAlgorithm,
    min_length: usize,
    max_length: usize,
    min_error: i32,
    max_error: i32,
) -> ValidationResult {
    // generate 2 strings
    let mut text = random_string(min_length, max_length);
    let mut query = mutate(&text, min_error, max_error);
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
    let (a_result, b_result) = match (a_type, b_type) {
        (AlignmentAlgorithm::Wavefront, AlignmentAlgorithm::Wavefront) => todo!(),
        (AlignmentAlgorithm::Wavefront, AlignmentAlgorithm::WavefrontAdaptive) => todo!(),
        (AlignmentAlgorithm::Wavefront, AlignmentAlgorithm::SWG) => (
            wavefront_align(&query, &text, &pens),
            affine_gap_align(&query, &text, &pens),
        ),
        (AlignmentAlgorithm::WavefrontAdaptive, AlignmentAlgorithm::Wavefront) => todo!(),
        (AlignmentAlgorithm::WavefrontAdaptive, AlignmentAlgorithm::WavefrontAdaptive) => todo!(),
        (AlignmentAlgorithm::WavefrontAdaptive, AlignmentAlgorithm::SWG) => todo!(),
        (AlignmentAlgorithm::SWG, AlignmentAlgorithm::Wavefront) => todo!(),
        (AlignmentAlgorithm::SWG, AlignmentAlgorithm::WavefrontAdaptive) => todo!(),
        (AlignmentAlgorithm::SWG, AlignmentAlgorithm::SWG) => todo!(),
    };

    match (a_result, b_result) {
        (AlignResult::Res(a), AlignResult::Res(b)) => {
            if a.score == b.score {
                ValidationResult::Passed
            } else {
                ValidationResult::Failed(ValidationFailureType::ScoreMismatch(ScoreMismatch {
                    query,
                    text,
                    a_score: a.score,
                    b_score: b.score,
                    query_aligned_a: a.query_aligned,
                    text_aligned_a: a.text_aligned,
                    query_aligned_b: b.query_aligned,
                    text_aligned_b: b.text_aligned,
                    pens,
                }))
            }
        }
        (AlignResult::Error(_), AlignResult::Error(_)) => ValidationResult::Passed,
        (AlignResult::Error(a), AlignResult::Res(_)) => ValidationResult::Failed(
            ValidationFailureType::AlignResultMismatch(AlignResultMismatch {
                failed_type: *b_type,
                failed: a,
            }),
        ),
        (AlignResult::Res(_), AlignResult::Error(a)) => ValidationResult::Failed(
            ValidationFailureType::AlignResultMismatch(AlignResultMismatch {
                failed_type: *a_type,
                failed: a,
            }),
        ),
    }
}
pub fn validate_sma(
    a_type: &AlignmentAlgorithm,
    min_length: usize,
    max_length: usize,
    min_error: i32,
    max_error: i32,
) -> (i32, Alignment, Penalties) {
    // generate 2 strings
    let mut text = random_string(min_length, max_length);
    let mut query = mutate(&text, min_error, max_error);
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
    let a_result = match a_type {
        AlignmentAlgorithm::Wavefront => wavefront_align(&query, &text, &pens),
        AlignmentAlgorithm::SWG => reference::affine_gap_align(&query, &text, &pens),
        _ => todo!(),
    };
    match a_result {
        AlignResult::Res(a) => (compute_score_from_alignment(&a, &pens), a, pens),
        AlignResult::Error(_) => panic!(),
    }
}
