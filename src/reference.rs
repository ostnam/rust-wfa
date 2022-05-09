use crate::alignment_lib::{Alignment, AlignmentError, AlignmentLayer, Penalties};
use std::cmp::min;

#[derive(Debug)]
struct AlignMat {
    inserts: Vec<Vec<(Option<i32>, Option<AlignmentLayer>)>>,
    matches: Vec<Vec<(Option<i32>, Option<AlignmentLayer>)>>,
    deletes: Vec<Vec<(Option<i32>, Option<AlignmentLayer>)>>,
}

pub fn affine_gap_align(a: &str, b: &str, pens: &Penalties) -> Result<Alignment, AlignmentError> {
    let align_mat = affine_gap_mat(a, b, pens);
    trace_back(&align_mat, a, b)
}

fn affine_gap_mat(a: &str, b: &str, pens: &Penalties) -> AlignMat {
    let mut result = new_mat(a, b, pens);
    let chars_a: Vec<char> = a.chars().collect();
    let chars_b: Vec<char> = b.chars().collect();
    for i in 1..chars_a.len() + 1 {
        for j in 1..chars_b.len() + 1 {
            result.inserts[i][j] = match (result.inserts[i - 1][j].0, result.matches[i - 1][j].0) {
                (Some(a), Some(b)) => {
                    if min(a + pens.extd_pen, b + pens.extd_pen + pens.open_pen)
                        == a + pens.extd_pen
                    {
                        (Some(a + pens.extd_pen), Some(AlignmentLayer::Inserts))
                    } else {
                        (
                            Some(b + pens.extd_pen + pens.open_pen),
                            Some(AlignmentLayer::Matches),
                        )
                    }
                }
                (Some(a), None) => (Some(a + pens.extd_pen), Some(AlignmentLayer::Inserts)),
                (None, Some(a)) => (
                    Some(a + pens.extd_pen + pens.open_pen),
                    Some(AlignmentLayer::Matches),
                ),
                (None, None) => panic!("(None, None), results.inserts"),
            };

            result.deletes[i][j] = match (result.deletes[i][j - 1].0, result.matches[i][j - 1].0) {
                (Some(a), Some(b)) => {
                    if min(a + pens.extd_pen, b + pens.extd_pen + pens.open_pen)
                        == a + pens.extd_pen
                    {
                        (Some(a + pens.extd_pen), Some(AlignmentLayer::Deletes))
                    } else {
                        (
                            Some(b + pens.extd_pen + pens.open_pen),
                            Some(AlignmentLayer::Matches),
                        )
                    }
                }
                (Some(a), None) => (Some(a + pens.extd_pen), Some(AlignmentLayer::Deletes)),
                (None, Some(a)) => (
                    Some(a + pens.extd_pen + pens.open_pen),
                    Some(AlignmentLayer::Matches),
                ),
                (None, None) => panic!("(None, None), results.deletes"),
            };

            let mismatch = if chars_a[i - 1] == chars_b[j - 1] {
                0
            } else {
                pens.mismatch_pen
            };

            result.matches[i][j] = match (
                result.matches[i - 1][j - 1].0,
                result.deletes[i][j].0,
                result.inserts[i][j].0,
            ) {
                (Some(a), Some(b), Some(c)) => {
                    if a + mismatch < b {
                        if a + mismatch < c {
                            (Some(a + mismatch), Some(AlignmentLayer::Matches))
                        } else {
                            (Some(c), Some(AlignmentLayer::Inserts))
                        }
                    } else if b <= c {
                        (Some(b), Some(AlignmentLayer::Deletes))
                    } else {
                        (Some(c), Some(AlignmentLayer::Inserts))
                    }
                }
                (Some(a), Some(b), None) => {
                    if a + mismatch < b {
                        (Some(a + mismatch), Some(AlignmentLayer::Matches))
                    } else {
                        (Some(b), Some(AlignmentLayer::Deletes))
                    }
                }
                (Some(a), None, Some(c)) => {
                    if a + mismatch < c {
                        (Some(a + mismatch), Some(AlignmentLayer::Matches))
                    } else {
                        (Some(c), Some(AlignmentLayer::Inserts))
                    }
                }
                (None, Some(b), Some(c)) => {
                    if b < c {
                        (Some(b), Some(AlignmentLayer::Deletes))
                    } else {
                        (Some(c), Some(AlignmentLayer::Inserts))
                    }
                }
                (Some(a), None, None) => (Some(a + mismatch), Some(AlignmentLayer::Matches)),
                (None, Some(b), None) => (Some(b), Some(AlignmentLayer::Deletes)),
                (None, None, Some(c)) => (Some(c), Some(AlignmentLayer::Inserts)),
                (None, None, None) => panic!("(None, None, None), result.matches"),
            };
        }
    }
    result
}

fn new_mat(a: &str, b: &str, pens: &Penalties) -> AlignMat {
    let a_length = a.len() + 1;
    let b_length = b.len() + 1;

    let mut inserts = vec![vec![(None, None); b_length]; a_length];
    let mut matches = vec![vec![(None, None); b_length]; a_length];
    let mut deletes = vec![vec![(None, None); b_length]; a_length];

    matches[0][0] = (Some(0), None);

    inserts[1][0] = (
        Some(pens.extd_pen + pens.open_pen),
        Some(AlignmentLayer::Matches),
    );
    matches[1][0] = inserts[1][0];
    for i in 2..a_length {
        inserts[i][0] = (
            Some(inserts[i - 1][0].0.unwrap() + pens.extd_pen),
            Some(AlignmentLayer::Inserts),
        );
        matches[i][0] = inserts[i][0];
    }

    deletes[0][1] = (
        Some(pens.extd_pen + pens.open_pen),
        Some(AlignmentLayer::Matches),
    );
    matches[0][1] = deletes[0][1];
    for i in 2..b_length {
        deletes[0][i] = (
            Some(deletes[0][i - 1].0.unwrap() + pens.extd_pen),
            Some(AlignmentLayer::Deletes),
        );
        matches[0][i] = deletes[0][i];
    }

    AlignMat {
        inserts,
        matches,
        deletes,
    }
}

fn trace_back(mat: &AlignMat, a: &str, b: &str) -> Result<Alignment, AlignmentError> {
    let mut result = Alignment {
        query_aligned: String::new(),
        text_aligned: String::new(),
        score: 0,
    };

    let mut a_pos = a.len();
    let mut b_pos = b.len();

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut layer = AlignmentLayer::Matches;
    result.score = mat.matches[a_pos][b_pos].0.unwrap();

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
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_align_score() {
        assert_eq!(
            affine_gap_align(
                "CAT",
                "CAT",
                &Penalties {
                    mismatch_pen: 1,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            Ok(Alignment {
                query_aligned: "CAT".to_string(),
                text_aligned: "CAT".to_string(),
                score: 0,
            })
        );
        assert_eq!(
            affine_gap_align(
                "CAT",
                "CATS",
                &Penalties {
                    mismatch_pen: 1,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            Ok(Alignment {
                query_aligned: "CAT-".to_string(),
                text_aligned: "CATS".to_string(),
                score: 2,
            })
        );
        assert_eq!(
            affine_gap_align(
                "XX",
                "YY",
                &Penalties {
                    mismatch_pen: 1,
                    extd_pen: 100,
                    open_pen: 100,
                }
            ),
            Ok(Alignment {
                query_aligned: "XX".to_string(),
                text_aligned: "YY".to_string(),
                score: 2,
            })
        );
        assert_eq!(
            affine_gap_align(
                "XX",
                "YY",
                &Penalties {
                    mismatch_pen: 100,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            Ok(Alignment {
                query_aligned: "XX--".to_string(),
                text_aligned: "--YY".to_string(),
                score: 6,
            })
        );
        assert_eq!(
            affine_gap_align(
                "XX",
                "YYYYYYYY",
                &Penalties {
                    mismatch_pen: 100,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            Ok(Alignment {
                query_aligned: "XX--------".to_string(),
                text_aligned: "--YYYYYYYY".to_string(),
                score: 12,
            })
        );
        assert_eq!(
            affine_gap_align(
                "XXZZ",
                "XXYZ",
                &Penalties {
                    mismatch_pen: 100,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            Ok(Alignment {
                query_aligned: "XX-ZZ".to_string(),
                text_aligned: "XXYZ-".to_string(),
                score: 4,
            })
        );
        assert_eq!(
            match affine_gap_align(
                "TCTTTACTCGCGCGTTGGAGAAATACAATAGT",
                "TCTATACTGCGCGTTTGGAGAAATAAAATAGT",
                &Penalties {
                    mismatch_pen: 1,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ) {
                Ok(s) => s.score,
                _ => -1,
            },
            6
        );

        assert_eq!(
            match affine_gap_align(
                "TCTTTACTCGCGCGTTGGAGAAATACAATAGT",
                "TCTATACTGCGCGTTTGGAGAAATAAAATAGT",
                &Penalties {
                    mismatch_pen: 135,
                    extd_pen: 19,
                    open_pen: 82,
                }
            ) {
                Ok(s) => s.score,
                _ => -1,
            },
            472
        );
    }
}
