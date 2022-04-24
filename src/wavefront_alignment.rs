/// This module defines:
/// * the types used for alignments
/// * the functions used for wavefront alignments
use super::alignment_lib::*;

/// This function is exported and can be called to perform an alignment.
/// The query cannot be longer than the text.
pub fn wavefront_align(query: &str, text: &str, pens: &Penalties) -> AlignResult {
    if query.is_empty() || text.is_empty() {
        return AlignResult::Error(AlignError::ZeroLength(format!(
            "At least one of the string slices passed to wavefront_align had a length of zero.
                        Length of query: {}
                        Length of text:  {}",
            query.len(),
            text.len()
        )));
    }
    if query.len() > text.len() {
        return AlignResult::Error(
                   AlignError::QueryTooLong(
                       "Query is longer than the reference string.
                        The length of the first string must be <= to the the length of the second string".to_string()
                      )
                  );
    }
    let mut current_front = new_wavefront_state(query, text, pens);
    loop {
        current_front.extend();
        if current_front.is_finished() {
            break;
        }
        current_front.increment_score();
        current_front.next();
    }
    current_front.backtrace()
}

/// Main struct, implementing the algorithm.
#[derive(Debug, PartialEq, Eq)]
struct WavefrontState<'a> {
    query: &'a str,
    text: &'a str,
    pens: &'a Penalties,
    q_chars: Vec<char>,
    t_chars: Vec<char>,

    /// Counter for looping and later backtracking.
    current_score: usize,

    grid: WavefrontGrid,

    /// Number of diagonals in the query-text alignment
    /// == to q_chars + t_chars - 1.
    num_diags: i32,

    /// The only diagonal on which we can align every char of query and
    /// text.
    final_diagonal: i32,

    /// Highest and lowest possible diags.
    highest_diag: i32,
    lowest_diag: i32,
}

/// Initializes a WavefrontState with the correct fields, for 2 string
/// slices and a penalties struct.
fn new_wavefront_state<'a>(
    query: &'a str,
    text: &'a str,
    pens: &'a Penalties,
) -> WavefrontState<'a> {
    let q_chars: Vec<char> = query.chars().collect();
    let t_chars: Vec<char> = text.chars().collect();

    let final_diagonal = (q_chars.len() as i32) - (t_chars.len() as i32); // A_k in the article
    let num_diags = (q_chars.len() + t_chars.len() + 1) as i32;
    let highest_diag = q_chars.len() as i32;
    let lowest_diag = 0 - t_chars.len() as i32;

    let mut matches = vec![vec![None; num_diags as usize]; 1];
    matches[0][(0 - lowest_diag) as usize] = Some((0, AlignmentLayer::Matches)); // Initialize the starting cell.

    let grid = new_wavefront_grid();

    WavefrontState {
        query,
        text,
        pens,
        q_chars,
        t_chars,
        current_score: 0,
        num_diags,
        final_diagonal,
        highest_diag,
        lowest_diag,
        grid,
    }
}

impl Wavefront for WavefrontState<'_> {
    fn extend(&mut self) {
        //! Extends the matches wavefronts to the furthest reaching point
        //! of the current score.
        let diag_range = self
            .grid
            .get_diag_range(self.current_score)
            .expect("get_diag_range returned None at wavefront_extend");

        for diag in (diag_range.0)..=(diag_range.1) {
            let text_pos = match self
                .grid
                .get(AlignmentLayer::Matches, self.current_score, diag)
            {
                Some((val, _)) => val,
                _ => continue,
            };
            let mut query_pos = (text_pos + diag) as usize;
            let mut text_pos = text_pos as usize;
            // The furthest reaching point value stored is the number
            // of matched chars in the Text string.
            // For any diagonal on the dynamic programming alignment
            // matrix, the number of chars matched for the Query is the
            // number of Text chars matched + diagonal.

            while query_pos < self.q_chars.len() && text_pos < self.t_chars.len() {
                match (
                    self.q_chars.get(query_pos as usize),
                    self.t_chars.get(text_pos as usize),
                ) {
                    (Some(q), Some(t)) => {
                        if q == t {
                            self.grid.increment(self.current_score, diag);
                            query_pos += 1;
                            text_pos += 1;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        }
    }

    fn increment_score(&mut self) {
        //! Increments the current score by 1.
        self.current_score += 1;
    }

    fn is_finished(&self) -> bool {
        //! Checks if the alignment is completed: for the current score,
        //! on the final diagonal, the furthest reaching point matches every
        //! char of Text and Query.
        match self.grid.get(
            AlignmentLayer::Matches,
            self.current_score,
            self.final_diagonal,
        ) {
            Some((score, _)) => score as usize >= self.t_chars.len(),
            _ => false,
        }
    }

    fn next(&mut self) {
        //! Equivalent of WAVEFRONT_NEXT

        // Calculating the next highest diagonal of the wavefront
        let mut hi = 1 + vec![
            self.current_score as i32 - self.pens.mismatch_pen,
            self.current_score as i32 - self.pens.open_pen - self.pens.extd_pen,
            self.current_score as i32 - self.pens.extd_pen,
        ]
        .into_iter()
        .filter(|x| *x >= 0)
        .map(|x| x as usize)
        .map(|x| self.grid.get_diag_range(x).unwrap_or(&(0, 0)).1)
        .max()
        .unwrap_or(0);

        if hi > self.highest_diag {
            hi = self.highest_diag;
        }

        let mut lo = vec![
            self.current_score as i32 - self.pens.mismatch_pen,
            self.current_score as i32 - self.pens.open_pen - self.pens.extd_pen,
            self.current_score as i32 - self.pens.extd_pen,
        ]
        .into_iter()
        .filter(|x| *x >= 0)
        .map(|x| x as usize)
        .map(|x| self.grid.get_diag_range(x).unwrap_or(&(0, 0)).0)
        .min()
        .unwrap_or(0)
            - 1;

        if lo < self.lowest_diag {
            lo = self.lowest_diag;
        }

        self.grid.add_layer(lo, hi);

        for diag in lo..=hi {
            self.update_ins(diag);
            self.update_del(diag);
            self.update_mat(diag);
        }
    }

    fn backtrace(&self) -> AlignResult {
        let mut curr_score = self.current_score;
        let mut curr_diag = self.final_diagonal;
        let mut curr_layer = AlignmentLayer::Matches;

        let mut query_aligned = String::new();
        let mut text_aligned = String::new();

        while curr_score > 0 {
            match &mut curr_layer {
                // If we're on a match
                AlignmentLayer::Matches => {
                    match self
                        .grid
                        .get(AlignmentLayer::Matches, curr_score, curr_diag)
                    {
                        Some((score, AlignmentLayer::Inserts)) => {
                            curr_layer = AlignmentLayer::Inserts;
                            let mut current_char = score;
                            while current_char
                                > self
                                    .grid
                                    .get(AlignmentLayer::Inserts, curr_score, curr_diag)
                                    .unwrap()
                                    .0
                            {
                                query_aligned
                                    .push(self.q_chars[(current_char + curr_diag - 1) as usize]);
                                text_aligned.push(self.t_chars[(current_char - 1) as usize]);
                                current_char -= 1;
                            }
                        }
                        Some((score, AlignmentLayer::Deletes)) => {
                            curr_layer = AlignmentLayer::Deletes;
                            let mut current_char = score;
                            while current_char
                                > self
                                    .grid
                                    .get(AlignmentLayer::Deletes, curr_score, curr_diag)
                                    .unwrap()
                                    .0
                            {
                                query_aligned
                                    .push(self.q_chars[(current_char + curr_diag - 1) as usize]);
                                text_aligned.push(self.t_chars[(current_char - 1) as usize]);
                                current_char -= 1;
                            }
                        }
                        Some((score, AlignmentLayer::Matches)) => {
                            let mut current_char = score;
                            curr_score -= self.pens.mismatch_pen as usize;
                            while current_char
                                > self
                                    .grid
                                    .get(AlignmentLayer::Matches, curr_score, curr_diag)
                                    .unwrap()
                                    .0
                            {
                                query_aligned
                                    .push(self.q_chars[(current_char + curr_diag - 1) as usize]);
                                text_aligned.push(self.t_chars[(current_char - 1) as usize]);
                                current_char -= 1;
                            }
                        }
                        _ => panic!(),
                    };
                }
                // If we're on the Inserts layer.
                AlignmentLayer::Inserts => {
                    match self
                        .grid
                        .get(AlignmentLayer::Inserts, curr_score, curr_diag)
                    {
                        Some((_, AlignmentLayer::Matches)) => {
                            let previous = self
                                .grid
                                .get(
                                    AlignmentLayer::Matches,
                                    curr_score - (self.pens.extd_pen + self.pens.open_pen) as usize,
                                    curr_diag - 1,
                                )
                                .unwrap();
                            query_aligned.push(self.q_chars[(previous.0 + curr_diag - 1) as usize]);
                            text_aligned.push('-');
                            curr_diag -= 1;
                            curr_score -= (self.pens.extd_pen + self.pens.open_pen) as usize;
                            curr_layer = AlignmentLayer::Matches;
                        }
                        Some((_, AlignmentLayer::Inserts)) => {
                            let previous = self
                                .grid
                                .get(
                                    AlignmentLayer::Inserts,
                                    curr_score - self.pens.extd_pen as usize,
                                    curr_diag - 1,
                                )
                                .unwrap();
                            query_aligned.push(self.q_chars[(previous.0 + curr_diag - 1) as usize]);
                            text_aligned.push('-');
                            curr_diag -= 1;
                            curr_score -= self.pens.extd_pen as usize;
                        }
                        _ => panic!(),
                    };
                }
                AlignmentLayer::Deletes => {
                    match self
                        .grid
                        .get(AlignmentLayer::Deletes, curr_score, curr_diag)
                    {
                        Some((_, AlignmentLayer::Matches)) => {
                            let previous = self
                                .grid
                                .get(
                                    AlignmentLayer::Matches,
                                    curr_score - (self.pens.extd_pen + self.pens.open_pen) as usize,
                                    curr_diag + 1,
                                )
                                .unwrap();
                            query_aligned.push('-');
                            text_aligned.push(self.t_chars[(previous.0) as usize]);
                            curr_diag += 1;
                            curr_score -= (self.pens.extd_pen + self.pens.open_pen) as usize;
                            curr_layer = AlignmentLayer::Matches;
                        }

                        Some((_, AlignmentLayer::Deletes)) => {
                            let previous = self
                                .grid
                                .get(
                                    AlignmentLayer::Deletes,
                                    curr_score - self.pens.extd_pen as usize,
                                    curr_diag + 1,
                                )
                                .unwrap();
                            query_aligned.push('-');
                            text_aligned.push(self.t_chars[(previous.0) as usize]);
                            curr_diag += 1;
                            curr_score -= self.pens.extd_pen as usize;
                        }
                        _ => panic!(),
                    };
                }
            };
        }
        if let AlignmentLayer::Matches = curr_layer {
            if curr_score == 0 {
                let remaining = self.grid.get(AlignmentLayer::Matches, 0, 0).unwrap().0 as usize;
                if remaining > 0 {
                    query_aligned =
                        query_aligned + &self.q_chars[..remaining].iter().rev().collect::<String>();
                    text_aligned =
                        text_aligned + &self.t_chars[..remaining].iter().rev().collect::<String>();
                }
            }
        }

        let q = query_aligned.chars().rev().collect();
        let t = text_aligned.chars().rev().collect();

        AlignResult::Res(Alignment {
            score: self.current_score as i32,
            query_aligned: q,
            text_aligned: t,
        })
    }
}

impl<'a> WavefrontState<'a> {
    fn update_ins(&mut self, diag: i32) {
        let from_open = if self.current_score >= (self.pens.open_pen + self.pens.extd_pen) as usize
        {
            self.grid.get(
                AlignmentLayer::Matches,
                self.current_score - (self.pens.open_pen + self.pens.extd_pen) as usize,
                diag - 1,
            )
        } else {
            None
        };
        let from_extd = if self.current_score >= self.pens.extd_pen as usize {
            self.grid.get(
                AlignmentLayer::Inserts,
                self.current_score - self.pens.extd_pen as usize,
                diag - 1,
            )
        } else {
            None
        };
        match (from_open, from_extd) {
            (None, None) => (),
            (Some(x), None) => {
                self.grid.set(
                    AlignmentLayer::Inserts,
                    self.current_score,
                    diag,
                    Some((x.0, AlignmentLayer::Matches)),
                );
            }
            (None, Some(x)) => {
                self.grid.set(
                    AlignmentLayer::Inserts,
                    self.current_score,
                    diag,
                    Some((x.0, AlignmentLayer::Inserts)),
                );
            }
            (Some(x), Some(y)) => {
                if x.0 > y.0 {
                    self.grid.set(
                        AlignmentLayer::Inserts,
                        self.current_score,
                        diag,
                        Some((x.0, AlignmentLayer::Matches)),
                    );
                } else {
                    self.grid.set(
                        AlignmentLayer::Inserts,
                        self.current_score,
                        diag,
                        Some((y.0, AlignmentLayer::Inserts)),
                    );
                }
            }
        }
    }

    fn update_del(&mut self, diag: i32) {
        let from_open = if self.current_score >= (self.pens.open_pen + self.pens.extd_pen) as usize
        {
            self.grid.get(
                AlignmentLayer::Matches,
                self.current_score - (self.pens.open_pen + self.pens.extd_pen) as usize,
                diag + 1,
            )
        } else {
            None
        };
        let from_extd = if self.current_score >= self.pens.extd_pen as usize {
            self.grid.get(
                AlignmentLayer::Deletes,
                self.current_score - self.pens.extd_pen as usize,
                diag + 1,
            )
        } else {
            None
        };

        match (from_open, from_extd) {
            (None, None) => (),
            (Some(x), None) => {
                self.grid.set(
                    AlignmentLayer::Deletes,
                    self.current_score,
                    diag,
                    Some((x.0 + 1, AlignmentLayer::Matches)),
                );
            }
            (None, Some(x)) => {
                self.grid.set(
                    AlignmentLayer::Deletes,
                    self.current_score,
                    diag,
                    Some((x.0 + 1, AlignmentLayer::Deletes)),
                );
            }
            (Some(x), Some(y)) => {
                if x.0 >= y.0 {
                    self.grid.set(
                        AlignmentLayer::Deletes,
                        self.current_score,
                        diag,
                        Some((x.0 + 1, AlignmentLayer::Matches)),
                    );
                } else {
                    self.grid.set(
                        AlignmentLayer::Deletes,
                        self.current_score,
                        diag,
                        Some((y.0 + 1, AlignmentLayer::Deletes)),
                    );
                }
            }
        }
    }

    fn update_mat(&mut self, diag: i32) {
        let from_mismatch = if self.current_score >= self.pens.mismatch_pen as usize {
            self.grid.get(
                AlignmentLayer::Matches,
                self.current_score - self.pens.mismatch_pen as usize,
                diag,
            )
        } else {
            None
        };

        self.grid.set(
            AlignmentLayer::Matches,
            self.current_score,
            diag,
            match (
                from_mismatch,
                self.grid
                    .get(AlignmentLayer::Inserts, self.current_score, diag),
                self.grid
                    .get(AlignmentLayer::Deletes, self.current_score, diag),
            ) {
                (None, None, None) => None,
                (Some(x), None, None) => Some((x.0 + 1, AlignmentLayer::Matches)),
                (None, Some(x), None) => Some((x.0, AlignmentLayer::Inserts)),
                (None, None, Some(x)) => Some((x.0, AlignmentLayer::Deletes)),
                (Some(x), Some(y), None) => Some(if x.0 + 1 >= y.0 {
                    (x.0 + 1, AlignmentLayer::Matches)
                } else {
                    (y.0, AlignmentLayer::Inserts)
                }),

                (Some(x), None, Some(y)) => Some(if x.0 + 1 >= y.0 {
                    (x.0 + 1, AlignmentLayer::Matches)
                } else {
                    (y.0, AlignmentLayer::Deletes)
                }),

                (None, Some(x), Some(y)) => Some(if x.0 > y.0 {
                    (x.0, AlignmentLayer::Inserts)
                } else {
                    (y.0, AlignmentLayer::Deletes)
                }),

                (Some(x), Some(y), Some(z)) => Some(if x.0 + 1 >= y.0 {
                    if x.0 + 1 >= z.0 {
                        (x.0 + 1, AlignmentLayer::Matches)
                    } else {
                        (z.0, AlignmentLayer::Deletes)
                    }
                } else if y.0 > z.0 {
                    (y.0, AlignmentLayer::Inserts)
                } else {
                    (z.0, AlignmentLayer::Deletes)
                }),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wavefront_state() {
        // Doesn't do much currently but at least if we accidently
        // change the behaviour/meaning of the wavefront state structs,
        // we'll notice.
        let state = new_wavefront_state(
            "GATA",
            "TAGAC",
            &Penalties {
                mismatch_pen: 1,
                open_pen: 2,
                extd_pen: 3,
            },
        );

        let mut manual_matches = vec![vec![None; 10]; 1];
        manual_matches[0][5] = Some((0, AlignmentLayer::Matches));
        let manual = WavefrontState {
            query: "GATA",
            text: "TAGAC",
            pens: &Penalties {
                mismatch_pen: 1,
                open_pen: 2,
                extd_pen: 3,
            },
            q_chars: "GATA".chars().collect(),
            t_chars: "TAGAC".chars().collect(),
            current_score: 0,
            num_diags: 10,
            final_diagonal: -1,
            highest_diag: 4,
            lowest_diag: -5,
            grid: new_wavefront_grid(),
        };

        assert_eq!(state, manual);
    }
    #[test]
    fn test_wavefront_update_ins() {
        // TODO
    }

    #[test]
    fn test_wavefront_update_mat() {
        // TODO
    }

    #[test]
    fn test_wavefront_backtrace() {
        // TODO
    }
    /*
    #[test]
    fn test_align_xx_yy() {
        // This case gave me a lot of trouble debugging, so I decided
        // to give it its own test function, to test it thoroughly.
        let mut wf = new_wavefront_state(
            "XX",
            "YY",
            &Penalties {
                mismatch_pen: 100,
                open_pen: 1,
                extd_pen: 1,
            },
        );
        wf.extend();
        assert_eq!(
            wf.matches[0],
            vec![None, None, Some((0, AlignmentLayer::Matches)), None, None]
        );
        assert_eq!(wf.inserts[0], vec![None, None, None, None, None]);
        assert_eq!(wf.deletes[0], vec![None, None, None, None, None]);

        wf.increment_score();
        wf.next();
        assert_eq!(wf.matches[1], vec![None, None, None, None, None]);
        assert_eq!(wf.inserts[1], vec![None, None, None, None, None]);
        assert_eq!(wf.deletes[1], vec![None, None, None, None, None]);

        wf.extend();
        wf.increment_score();
        wf.next();
        assert_eq!(
            wf.matches[2],
            vec![
                None,
                Some((1, AlignmentLayer::Deletes)),
                None,
                Some((0, AlignmentLayer::Inserts)),
                None
            ]
        );
        assert_eq!(
            wf.inserts[2],
            vec![None, None, None, Some((0, AlignmentLayer::Matches)), None]
        );
        assert_eq!(
            wf.deletes[2],
            vec![None, Some((1, AlignmentLayer::Matches)), None, None, None]
        );

        wf.extend();
        wf.increment_score();
        wf.next();
        assert_eq!(
            wf.matches[3],
            vec![
                Some((2, AlignmentLayer::Deletes)),
                None,
                None,
                None,
                Some((0, AlignmentLayer::Inserts))
            ]
        );
        assert_eq!(
            wf.inserts[3],
            vec![None, None, None, None, Some((0, AlignmentLayer::Inserts))]
        );
        assert_eq!(
            wf.deletes[3],
            vec![Some((2, AlignmentLayer::Deletes)), None, None, None, None]
        );

        wf.extend();
        wf.increment_score();
        wf.next();
        assert_eq!(
            wf.matches[4],
            vec![
                Some((2, AlignmentLayer::Deletes)),
                None,
                Some((1, AlignmentLayer::Deletes)),
                None,
                Some((0, AlignmentLayer::Inserts))
            ]
        );
        assert_eq!(
            wf.inserts[4],
            vec![
                None,
                None,
                Some((1, AlignmentLayer::Matches)),
                None,
                Some((0, AlignmentLayer::Matches))
            ]
        );
        assert_eq!(
            wf.deletes[4],
            vec![
                Some((2, AlignmentLayer::Matches)),
                None,
                Some((1, AlignmentLayer::Matches)),
                None,
                None
            ]
        );

        wf.extend();
        wf.increment_score();
        wf.next();
        assert_eq!(
            wf.matches[5],
            vec![
                None,
                Some((2, AlignmentLayer::Deletes)),
                None,
                Some((1, AlignmentLayer::Deletes)),
                None
            ]
        );
        assert_eq!(
            wf.inserts[5],
            vec![
                None,
                Some((2, AlignmentLayer::Matches)),
                None,
                Some((1, AlignmentLayer::Inserts)),
                None
            ]
        );
        assert_eq!(
            wf.deletes[5],
            vec![
                None,
                Some((2, AlignmentLayer::Deletes)),
                None,
                Some((1, AlignmentLayer::Matches)),
                None
            ]
        );

        assert!(!wf.is_finished());
        wf.extend();
        wf.increment_score();
        wf.next();
        assert_eq!(
            wf.matches[6],
            vec![
                Some((3, AlignmentLayer::Deletes)),
                Some((2, AlignmentLayer::Deletes)),
                Some((2, AlignmentLayer::Deletes)),
                Some((1, AlignmentLayer::Deletes)),
                Some((1, AlignmentLayer::Inserts))
            ]
        );
        assert_eq!(
            wf.inserts[6],
            vec![
                None,
                Some((2, AlignmentLayer::Matches)),
                Some((2, AlignmentLayer::Inserts)),
                Some((1, AlignmentLayer::Matches)),
                Some((1, AlignmentLayer::Inserts))
            ]
        );
        assert_eq!(
            wf.deletes[6],
            vec![
                Some((3, AlignmentLayer::Deletes)),
                Some((2, AlignmentLayer::Matches)),
                Some((2, AlignmentLayer::Deletes)),
                Some((1, AlignmentLayer::Matches)),
                None
            ]
        );
        assert!(wf.is_finished());
    }
    */

    #[test]
    fn test_align_avd() {
        assert_eq!(
            wavefront_align(
                "AViidI",
                "ViidIM",
                &Penalties {
                    mismatch_pen: 3,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            AlignResult::Res(Alignment {
                query_aligned: "AViidI-".to_string(),
                text_aligned: "-ViidIM".to_string(),
                score: 4,
            })
        );

        assert_eq!(
            wavefront_align(
                "AVD",
                "VDM",
                &Penalties {
                    mismatch_pen: 2,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            AlignResult::Res(Alignment {
                query_aligned: "AVD-".to_string(),
                text_aligned: "-VDM".to_string(),
                score: 4,
            })
        );

        assert_eq!(
            wavefront_align(
                "AV",
                "VM",
                &Penalties {
                    mismatch_pen: 2,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            AlignResult::Res(Alignment {
                query_aligned: "AV".to_string(),
                text_aligned: "VM".to_string(),
                score: 4,
            })
        );
    }

    #[test]
    fn test_wavefront_align() {
        assert_eq!(
            wavefront_align(
                "CAT",
                "CAT",
                &Penalties {
                    mismatch_pen: 1,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            AlignResult::Res(Alignment {
                query_aligned: "CAT".to_string(),
                text_aligned: "CAT".to_string(),
                score: 0,
            })
        );
        assert_eq!(
            wavefront_align(
                "CAT",
                "CATS",
                &Penalties {
                    mismatch_pen: 1,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            AlignResult::Res(Alignment {
                query_aligned: "CAT-".to_string(),
                text_aligned: "CATS".to_string(),
                score: 2,
            })
        );
        assert_eq!(
            wavefront_align(
                "XX",
                "YY",
                &Penalties {
                    mismatch_pen: 1,
                    extd_pen: 100,
                    open_pen: 100,
                }
            ),
            AlignResult::Res(Alignment {
                query_aligned: "XX".to_string(),
                text_aligned: "YY".to_string(),
                score: 2,
            })
        );
        assert_eq!(
            wavefront_align(
                "XX",
                "YY",
                &Penalties {
                    mismatch_pen: 100,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            AlignResult::Res(Alignment {
                query_aligned: "XX--".to_string(),
                text_aligned: "--YY".to_string(),
                score: 6,
            })
        );
        assert_eq!(
            wavefront_align(
                "XX",
                "YYYYYYYY",
                &Penalties {
                    mismatch_pen: 100,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            AlignResult::Res(Alignment {
                query_aligned: "XX--------".to_string(),
                text_aligned: "--YYYYYYYY".to_string(),
                score: 12,
            })
        );
        assert_eq!(
            wavefront_align(
                "XXZZ",
                "XXYZ",
                &Penalties {
                    mismatch_pen: 100,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ),
            AlignResult::Res(Alignment {
                query_aligned: "XX-ZZ".to_string(),
                text_aligned: "XXYZ-".to_string(),
                score: 4,
            })
        );
    }

    #[test]
    fn assert_align_score() {
        assert_eq!(
            match wavefront_align(
                "TCTTTACTCGCGCGTTGGAGAAATACAATAGT",
                "TCTATACTGCGCGTTTGGAGAAATAAAATAGT",
                &Penalties {
                    mismatch_pen: 1,
                    extd_pen: 1,
                    open_pen: 1,
                }
            ) {
                AlignResult::Res(s) => s.score,
                _ => -1,
            },
            6
        );

        assert_eq!(
            match wavefront_align(
                "TCTTTACTCGCGCGTTGGAGAAATACAATAGT",
                "TCTATACTGCGCGTTTGGAGAAATAAAATAGT",
                &Penalties {
                    mismatch_pen: 135,
                    extd_pen: 19,
                    open_pen: 82,
                }
            ) {
                AlignResult::Res(s) => s.score,
                _ => -1,
            },
            472
        );
    }
}
