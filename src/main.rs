use wavefront_align::wavefront::*;
use std::io::{stdin, BufRead};

fn main() {
    let mut query: String = String::new();
    let mut text: String = String::new();

    stdin().lock().read_line(&mut query).unwrap();
    stdin().lock().read_line(&mut text).unwrap();

    query = query.trim().to_string();
    text = text.trim().to_string();
    
    let pens = Penalties {
        match_pen: 0,
        mismatch_pen: 1,
        open_pen:  3,
        extd_pen:  2,
    };

    match wavefront_align(&query, &text, &pens) {
        Ok(alignment) => print!("{}\n{}\n{}\n", alignment.score, alignment.query_aligned, alignment.text_aligned),
        Err(e) => panic!("Alignment returned an error: {:?}", e),
    };
}
