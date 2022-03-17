use clap::Parser;
use lib::{reference::affine_gap_align, validation_lib::AlignmentType, wavefront_alignment};
use std::io::{stdin, BufRead};
use std::time::Instant;

/// Struct used for parsing CLI args with clap.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct MainArgs {
    #[clap(short, long)]
    method: AlignmentType,

    #[clap(short, long)]
    mismatch_pen: i32,
    #[clap(short, long)]
    open_pen: i32,
    #[clap(short, long)]
    extd_pen: i32,

    #[clap(short, long)]
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

    let alignment = match args.method {
        AlignmentType::WavefrontNaive => wavefront_alignment::wavefront_align(&query, &text, &pens),
        AlignmentType::WavefrontNaiveAdaptive => {
            wavefront_alignment::wavefront_align_adaptive(&query, &text, &pens)
        }
        AlignmentType::Reference => affine_gap_align(&query, &text, &pens),
    };

    if let Some(t) = before {
        let elapsed = t.elapsed();
        println!("Aligned in {:.2?}", elapsed);
    };

    match alignment {
        lib::alignment_lib::AlignResult::Res(alignment) => print!(
            "{}\n{}\n{}\n",
            alignment.score, alignment.query_aligned, alignment.text_aligned
        ),
        lib::alignment_lib::AlignResult::Error(e) => panic!("Alignment returned an error: {:?}", e),
    };
}
