use lib::{
    alignment_lib::Penalties, reference::affine_gap_align, wavefront_alignment::wavefront_align,
};

fn main() {
    use std::time::Instant;
    let a = "ATGCACGGTGGTAGGTGCGGTCCCGCTGAGAGACTCACCTATGTGCACTATACCCCACAGGTTAGCATGTTTGGGCTGTTGGTACCCCTGACCTCTAGTTCGGACAGATCTGATGCAAGTGGCACGTGATCGCATGCCTATTGTGAATCTGTCAAGGACTTTGCTCTATTAGGTGATTTAGGATGAGCCACATACTGCTCAAAGTGTTGTACCCCACTGGAATTAAGTCGACGCAGAACGTCTCTAGCAAAACTAGTTCCCGTATTGTTAGTGCACGGGTTGACCATCTCGATCCTCTGATAACGAGCTAATTAAACACACTCACGTGGGTCTTTCAGTATTTGAAGCAAGGATGCGGTACTGAGCGGTCTTCGCGTATAACGCCCTTAGACGGGAACTAGGTACAGCCGCGATTCCGCCCCTTACACGTCCGAGACCCTTTCTTAAATGGAACAATACGATTGACTGGACGAGCTGGTGCGGCTTGTTACTTCGTGCCCGTAGCCGAAGGCACTAACTTCCTCGCCCTTTGGTCTCTAGCGTGTTGTAGTGTGAGACTACGGTCATTATCCGCAGATCCGCACAAACGTCTCATTAGGCGAGACTGCGAGAGGGGTGACGCATGTTTGGACGTGTGCCCCTCAGATCGTGACTAATATGGATCTCCGGGTAGAAACAGTTCACCACACCGACGCATGGGAGGGTTTATGTTTACACGCGTAGAGTCCGATTGGGGCCGCAGCAGGATCGCCGAGCATGGAGGTCACTGTCGCGGACTTGGCACGGGGGCCGATGCATAGGCCAAAACTTACCCGTTCGGGTAGTTTTGAGACATCCCTGAGGCAATACGTACTGGCTACCTAGAAACCTACCTTGTTGCCGTCCCTCGAGCCAGCCATAACGTAATTTCAGTATCCGGATAGACACGAAGACAAGCAGAGTGCTGGCCCCACTCGTTATATAGACACGGGACCCCTAGCCGTGTCCGGCG";
    let b = "ATGCACGGTGGTAGGTGCGGTCCCGCTGAGGAGACTCACCGTATGTGCACTATACCCCACAGGTTAGCATGTTTGGGCTGTTGGTAACCCTGACCTCTAGTTCGGACAGATCTGATGCAAGTGGCACGTGATCGCATGCCTATTGTGAATCTGTCAAGGACTTTGCTCTATTAGGTGATTTAGGATGAGCCACATACTGCTCAAAGTGTTGTACCCCACTGGAATTAAGTCGACGCAGAACGTCTCTAGCAAAACTAGTTCCCGTATTGTTAGTGCACGGGTTGACCATCTCGATCCTCTGATAACGAGCTAATTAAACACACTCACGTGGGTCTTTCAGTATTTGAAGCAAGGATGCGGTACTGAGCGGTCTTCGCGTATAACGCCCTTAGACGGGAACTAGGTACAGCCGCGATTCCGCCCCTTACACGTCCGAGACCCTTTCTTAAATGGAACAATACGATTGACTGGACGAGCTGGTGCGGCTTGTTACTTCGTGCCCGTAGCCGAAGGCACTAACTTCCTCGCCCTTTGGTCTCTAGCGTGTTGTAGTGTGAGACTACGGTCATTATCCGCAGATCCGCACAAACGTCTCATTAGGCGAGACTGCGAGAGGGGTGACGCATGTTTGGACGTGTGCCCCTCAGATCGTGACTAATATGGATCTCCGGGTAGAAACAGTTCACCACACCGACGCATGGGAGGGTTTATGTTTACACGCGTAGAGTCCGATTGGGGCCGCAGCAGGATCGCCGAGCATGGAGGTCACTGTCGCGGACTTGGCACGGGGGCCGATGCATAGGCCAAAACTTACCCGTTCGGGTAGTTTTGAGAGTCGTCATCCCTGAGGCAATACGTACTGGCTACCTAGAAACCTACCTTGTTGCCGGTCCCTCGAGCCAGCCATAACGTAATTTCAGTATCCGGATAGACACGAAGACAAGCAGAGTGCTGGCCCCACTCGTTATATAGACACGGGACCCCTAGCCTGTGTCCGGCG";
    let now = Instant::now();
    affine_gap_align(
        a,
        b,
        &Penalties {
            mismatch_pen: 6,
            open_pen: 2,
            extd_pen: 4,
        },
    );

    let t = now.elapsed();
    println!("Time to align using SWG alignment: {:.2?}", t);

    let now = Instant::now();
    wavefront_align(
        a,
        b,
        &Penalties {
            mismatch_pen: 6,
            open_pen: 2,
            extd_pen: 4,
        },
    );

    let t = now.elapsed();
    println!("Time to align using wavefront_align: {:.2?}", t);

}
