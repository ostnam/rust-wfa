use std::io::{stdin, BufRead};

fn main() {
    let mut query: String = String::new();
    let mut text: String = String::new();

    stdin().lock().read_line(&mut query).unwrap();
    stdin().lock().read_line(&mut text).unwrap();

    query = query.trim().to_string();
    text = text.trim().to_string();
    
    let pens = wavefront_align::alignment_lib::Penalties {
        mismatch_pen: 1,
        open_pen:  1,
        extd_pen:  1,
    };

    match wavefront_align::wavefront::wavefront_align(&query, &text, &pens) {
        wavefront_align::alignment_lib::AlignResult::Res(alignment) => print!("{}\n{}\n{}\n", alignment.score, alignment.query_aligned, alignment.text_aligned),
        wavefront_align::alignment_lib::AlignResult::Error(e) => panic!("Alignment returned an error: {:?}", e),
    };
}
