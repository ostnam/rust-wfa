use clap::Parser;
use lib::{reference::affine_gap_align, alignment_lib::AlignmentAlgorithm, wavefront_alignment};
use std::io::{stdin, BufRead};
use std::time::Instant;

/// Struct used for parsing CLI args with clap.
#[derive(Parser, Debug)]
#[clap(author="Mansour Tsougaev", version, about="Wavefront alignment in Rust.")]
struct MainArgs {
    #[clap(short, long, default_value_t = AlignmentAlgorithm::Wavefront)]
    /// Alignment algorithm that will be used. Possible values: Wavefront, SWG.
    algorithm: AlignmentAlgorithm,

    #[clap(short, long)]
    /// Penalty for mismatching 2 chars.
    mismatch_pen: i32,

    #[clap(short, long)]
    /// Penalty for opening a gap.
    open_pen: i32,

    #[clap(short, long)]
    /// Penalty for extending a gap by 1. Is also applied once when the gap is opened.
    extd_pen: i32,

    #[clap(short, long)]
    /// Whether to print how long it took to align.
    bench: bool,
}

fn main() {
    // parse CLI args
    let args = MainArgs::parse();

    // read alignment strings from stdin
    let mut query: String = String::new();
    let mut text: String = String::new();
    stdin().lock().read_line(&mut query).unwrap();
    stdin().lock().read_line(&mut text).unwrap();
    query = query.trim().to_string();
    text = text.trim().to_string();

    let pens = lib::alignment_lib::Penalties {
        mismatch_pen: args.mismatch_pen,
        open_pen: args.open_pen,
        extd_pen: args.extd_pen,
    };

    let before = if args.bench {
        Some(Instant::now())
    } else {
        None
    };

    let alignment = match args.algorithm {
        AlignmentAlgorithm::Wavefront => wavefront_alignment::wavefront_align(&query, &text, &pens),
        AlignmentAlgorithm::WavefrontAdaptive => {
            panic!("WFA-adaptive not yet implemented.");
        }
        AlignmentAlgorithm::SWG => affine_gap_align(&query, &text, &pens),
    };

    if let Some(t) = before {
        let elapsed = t.elapsed();
        println!("Aligned in {:.2?}", elapsed);
    };

    match alignment {
        Ok(alignment) => print!(
            "{}\n{}\n{}\n",
            alignment.score, alignment.query_aligned, alignment.text_aligned
        ),
        Err(e) => panic!("Alignment returned an error: {:?}", e),
    };
}
