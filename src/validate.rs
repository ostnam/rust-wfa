use lib::validation_lib::*;

fn main() {
    for cycle in 0..u32::MAX {
        match compare_alignment(&AlignmentType::WavefrontNaive,
                                    &AlignmentType::Reference,
                                    2,
                                    5,
                                    0,
                                    50,) {
            ValidationResult::Passed => println!("Validation successful at cycle {}", cycle),
            ValidationResult::Failed(a) => {
                println!("Validation failed at cycle {}. \n {:?}", cycle, a);
                return ();
            }
        }
    }
}
