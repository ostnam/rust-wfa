use std::cmp::max;

pub use crate::alignment_lib::{self, Penalties, AlignmentLayer};

fn max3<T: Ord>(a: T, b: T, c: T) -> T {
    max(a, max(b, c))
}

#[derive(Debug)]
struct AlignMat {
    inserts: Vec<Vec<(i32, Option<alignment_lib::AlignmentLayer>)>>,
    matches: Vec<Vec<(i32, Option<alignment_lib::AlignmentLayer>)>>,
    deletes: Vec<Vec<(i32, Option<alignment_lib::AlignmentLayer>)>>,
}

pub fn affine_gap_align(a: &str, b: &str, pens: &alignment_lib::Penalties) -> alignment_lib::AlignResult {
    let align_mat = affine_gap_mat(a, b, pens);
    trace_back(&align_mat, a, b)
}

fn affine_gap_mat(a: &str, b: &str, pens: &Penalties) -> AlignMat {
    let mut result = new_mat(a, b);
    let chars_a: Vec<char> = a.chars().collect();
    let chars_b: Vec<char> = b.chars().collect();

    for i in 1..chars_a.len() + 1 {
        for j in 1..chars_b.len() + 1 {
            result.inserts[i][j] = if max(
                 result.inserts[i - 1][j].0 - pens.extd_pen,
                 result.matches[i - 1][j].0 - pens.extd_pen - pens.open_pen,
            ) == result.inserts[i - 1][j].0 - pens.extd_pen
            {
                (result.inserts[i - 1][j].0 - pens.extd_pen, Some(AlignmentLayer::Inserts))
            } else {
                (result.matches[i - 1][j].0 - pens.open_pen - pens.extd_pen, Some(AlignmentLayer::Matches))
            };

            result.deletes[i][j] = if max(
                 result.deletes[i][j - 1].0 - pens.extd_pen,
                 result.matches[i][j - 1].0 - pens.extd_pen - pens.open_pen,
            ) == result.deletes[i][j - 1].0 - pens.extd_pen
            {
                (result.deletes[i][j - 1].0 - pens.extd_pen, Some(AlignmentLayer::Deletes))
            } else {
                (result.matches[i][j - 1].0 - pens.open_pen - pens.extd_pen, Some(AlignmentLayer::Matches))
            };

            let mismatch = if chars_a[i - 1] == chars_b[j - 1] {
                0 
            } else {
                pens.mismatch_pen 
            };

            result.matches[i][j] = if max3(
                 result.matches[i - 1][j - 1].0 - mismatch,
                 result.deletes[i][j].0,
                 result.inserts[i][j].0,
            ) == result.matches[i - 1][j - 1].0 - mismatch {
                (
                    result.matches[i - 1][j - 1].0 - mismatch,
                    Some(AlignmentLayer::Matches),
                )
            } else if result.deletes[i][j].0 >= result.inserts[i][j].0 {
                (result.deletes[i][j].0, Some(AlignmentLayer::Deletes))
            } else {
                (result.inserts[i][j].0, Some(AlignmentLayer::Inserts))
            };
        }
    }
    result
}

fn new_mat(a: &str, b: &str) -> AlignMat {
    let a_length = a.len() + 1;
    let b_length = b.len() + 1;

    AlignMat {
        inserts: vec![vec![(0, None); b_length]; a_length],
        matches: vec![vec![(0, None); b_length]; a_length],
        deletes: vec![vec![(0, None); b_length]; a_length],
    }
}

fn trace_back(mat: &AlignMat, a: &str, b: &str) -> alignment_lib::AlignResult {
    let mut result = alignment_lib::Alignment {
        query_aligned: String::new(),
        text_aligned: String::new(),
        score: 0,
    };

    let mut a_pos = a.len();
    let mut b_pos = b.len();

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut layer = AlignmentLayer::Matches;
    result.score = 0 - mat.matches[a_pos][b_pos].0;

    while (a_pos > 0) || (b_pos > 0) {
        if a_pos == 0 {
            b_pos -= 1;
            result.query_aligned.push('-');
            result.text_aligned.push(b_chars[b_pos]);
        } else if b_pos == 0 {
            a_pos -= 1;
            result.query_aligned.push(a_chars[a_pos]);
            result.text_aligned.push('-');
        } else {
            match &mut layer {
                AlignmentLayer::Inserts => {
                    result.query_aligned.push(a_chars[a_pos - 1]);
                    result.text_aligned.push('-');
                    if let Some(AlignmentLayer::Matches) = mat.inserts[a_pos][b_pos].1 {
                        layer = AlignmentLayer::Matches;
                    };
                    a_pos -= 1;
                }
                AlignmentLayer::Matches => match mat.matches[a_pos][b_pos].1 {
                    Some(AlignmentLayer::Matches) => {
                        a_pos -= 1;
                        b_pos -= 1;
                        result.query_aligned.push(a_chars[a_pos]);
                        result.text_aligned.push(b_chars[b_pos]);
                    }
                    Some(AlignmentLayer::Inserts) => {
                        layer = AlignmentLayer::Inserts;
                    }
                    Some(AlignmentLayer::Deletes) => {
                        layer = AlignmentLayer::Deletes;
                    }
                    _ => panic!(),
                },
                AlignmentLayer::Deletes => {
                    result.query_aligned.push('-');
                    result.text_aligned.push(b_chars[b_pos - 1]);
                    if let Some(AlignmentLayer::Matches) = mat.deletes[a_pos][b_pos].1 {
                        layer = AlignmentLayer::Matches;
                    };
                    b_pos -= 1;
                }
            }
        }
    }
    result.query_aligned = result.query_aligned.chars().rev().collect();
    result.text_aligned = result.text_aligned.chars().rev().collect();
    alignment_lib::AlignResult::Res(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_align_score() {
        assert_eq!(affine_gap_align("CAT", "CAT",
                   &Penalties {
                       mismatch_pen: 1,
                       extd_pen: 1,
                       open_pen: 1,
                   }),
                   alignment_lib::AlignResult::Res(alignment_lib::Alignment {
                       query_aligned: "CAT".to_string(),
                       text_aligned: "CAT".to_string(),
                       score: 0,
                   }
               )
       );
       assert_eq!(affine_gap_align("CAT", "CATS",
                   &Penalties {
                       mismatch_pen: 1,
                       extd_pen: 1,
                       open_pen: 1,
                   }),
                   alignment_lib::AlignResult::Res(alignment_lib::Alignment {
                       query_aligned: "CAT-".to_string(),
                       text_aligned: "CATS".to_string(),
                       score: 2,
                   }
               )
       );
       assert_eq!(affine_gap_align("XX", "YY",
                   &Penalties {
                       mismatch_pen: 1,
                       extd_pen: 100,
                       open_pen: 100,
                   }),
                   alignment_lib::AlignResult::Res(alignment_lib::Alignment {
                       query_aligned: "XX".to_string(),
                       text_aligned: "YY".to_string(),
                       score: 2,
                   }
               )
       );
       assert_eq!(affine_gap_align("XX", "YY",
                   &Penalties {
                       mismatch_pen: 100,
                       extd_pen: 1,
                       open_pen: 1,
                   }),
                   alignment_lib::AlignResult::Res(alignment_lib::Alignment {
                       query_aligned: "XX--".to_string(),
                       text_aligned: "--YY".to_string(),
                       score: 6,
                   }
               )
       );
       assert_eq!(affine_gap_align("XX", "YYYYYYYY",
                   &Penalties {
                       mismatch_pen: 100,
                       extd_pen: 1,
                       open_pen: 1,
                   }),
                   alignment_lib::AlignResult::Res(alignment_lib::Alignment {
                       query_aligned: "XX--------".to_string(),
                       text_aligned: "--YYYYYYYY".to_string(),
                       score: 12,
                   }
               )
       );
        assert_eq!(affine_gap_align("XXZZ", "XXYZ",
                       &Penalties {
                           mismatch_pen: 100,
                           extd_pen: 1,
                           open_pen: 1,
                       }),
                       alignment_lib::AlignResult::Res(alignment_lib::Alignment {
                           query_aligned: "XX-ZZ".to_string(),
                           text_aligned:  "XXYZ-".to_string(),
                           score: 4,
                       }
                   )
           );
    }
}
