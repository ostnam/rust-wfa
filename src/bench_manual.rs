use lib::{
    alignment_lib::Penalties, reference::affine_gap_align, wavefront_alignment::wavefront_align,
};
use std::io::{stdin, BufRead};
use std::time::Instant;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    mismatch_pen: i32,
    opening_pen: i32,
    extension_pen: i32,
}

fn main() {
    let args = Args::parse();
    let mut query = String::new();
    let mut text = String::new();
    stdin().lock().read_line(&mut query).unwrap();
    stdin().lock().read_line(&mut text).unwrap();
    let now = Instant::now();
    affine_gap_align(
        &query,
        &text,
        &Penalties {
            mismatch_pen: args.mismatch_pen,
            open_pen: args.opening_pen,
            extd_pen: args.extension_pen,
        },
    );
    let t = now.elapsed();
    println!("Time to align using SWG alignment: {:.2?}", t);

    let now = Instant::now();
    wavefront_align(
        &query,
        &text,
        &Penalties {
            mismatch_pen: args.mismatch_pen,
            open_pen: args.opening_pen,
            extd_pen: args.extension_pen,
        },
    );

    let t = now.elapsed();
    println!("Time to align using wavefront_align: {:.2?}", t);
}
