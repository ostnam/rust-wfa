use crate::alignment_lib::{self, Penalties};
use crate::reference::affine_gap_align;
use crate::wavefront_alignment::wavefront_align;
use rand::{thread_rng, Rng};
use rand::distributions::{Alphanumeric, Standard, Distribution};

pub enum AlignmentType {
    WavefrontNaive,
    WavefrontNaiveAdaptive,
    Reference,
}

enum MutationType {
    Insertion,
    Deletion,
    Substitution,
}

impl Distribution<MutationType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MutationType {
        match rng.gen_range(0..=2) {
            0 => MutationType::Insertion,
            1 => MutationType::Deletion,
            _ => MutationType::Substitution,
        }
    }
}

fn random_string(min_length: usize, max_length: usize)  -> String {
    let mut rng = thread_rng();
    let length = rng.gen_range(min_length..max_length);

    (&mut rng).sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn gen_new_char() -> char {
    let mut rng = thread_rng();
    (&mut rng).sample_iter(Alphanumeric)
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

pub fn compare_alignment(a_type: &AlignmentType, b_type: &AlignmentType, min_length: usize, max_length: usize, min_error: i32, max_error: i32) -> (alignment_lib::AlignResult, alignment_lib::AlignResult) {
    // generate 2 strings
    let mut text = random_string(min_length, max_length);
    let mut query = mutate(&text, min_error, max_error);
    if query.len() > text.len() {
        std::mem::swap(&mut query, &mut text);
    }

    // generate pens
    let mut rng = thread_rng();

    let pens = Penalties {
        mismatch_pen: rng.gen_range(0..100),
        open_pen: rng.gen_range(0..100),
        extd_pen: rng.gen_range(0..100),
    };
    
    
    // align them using the method
    match (a_type, b_type) {
        (AlignmentType::WavefrontNaive, AlignmentType::WavefrontNaive) => todo!(),
        (AlignmentType::WavefrontNaive, AlignmentType::WavefrontNaiveAdaptive) => todo!(),
        (AlignmentType::WavefrontNaive, AlignmentType::Reference) => (wavefront_align(&query, &text, &pens), affine_gap_align(&query, &text, &pens)),
        (AlignmentType::WavefrontNaiveAdaptive, AlignmentType::WavefrontNaive) => todo!(),
        (AlignmentType::WavefrontNaiveAdaptive, AlignmentType::WavefrontNaiveAdaptive) => todo!(),
        (AlignmentType::WavefrontNaiveAdaptive, AlignmentType::Reference) => todo!(),
        (AlignmentType::Reference, AlignmentType::WavefrontNaive) => todo!(),
        (AlignmentType::Reference, AlignmentType::WavefrontNaiveAdaptive) => todo!(),
        (AlignmentType::Reference, AlignmentType::Reference) => todo!(),
    }
}
