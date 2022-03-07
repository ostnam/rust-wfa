use lib::validation_lib::*;

fn main() {
    for cycle in 0..u32::MAX {
        let res = compare_alignment(&AlignmentType::WavefrontNaive,
                                    &AlignmentType::Reference,
                                    10,
                                    100,
                                    0,
                                    50,);

        match (res.0, res.1) {
            (lib::alignment_lib::AlignResult::Res(a), lib::alignment_lib::AlignResult::Res(b)) => if a.score == b.score {
            println!("Successfully aligned another pair of strings at cycle {}", cycle);
            } else {
            panic!("Alignment failure at cycle {}, \n Wavefront alignment: {:?} \n Reference alignment: {:?}", cycle, a, b);
            },
            (lib::alignment_lib::AlignResult::Res(_), lib::alignment_lib::AlignResult::Error(_)) => todo!(),
            (lib::alignment_lib::AlignResult::Error(_), lib::alignment_lib::AlignResult::Res(_)) => todo!(),
            (lib::alignment_lib::AlignResult::Error(_), lib::alignment_lib::AlignResult::Error(_)) => todo!(),
        }
    }
}
