/// This module defines all the types and functions used in the crate.
pub mod wavefront {
    use std::cmp::{min, max};

    /// This function is exported and can be called to perform an alignment.
    /// The query cannot be longer than the text.
    pub fn wavefront_align(query: &str, text: &str, pens: &Penalties) 
        -> AlignResult {
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

    /// This function is exported and can be called to perform an alignment.
    /// The query cannot be longer than the text.
    pub fn wavefront_align_adaptive(query: &str,
                                    text: &str,
                                    pens: &Penalties) 
        -> AlignResult {
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
            current_front.extend();                // WF-extend
            if current_front.is_finished() {
                break;
            }
            current_front.increment_score();       // Add 1 to the score.
            current_front.next();                  // WF-next
        }
        current_front.backtrace()
    }


    /// Holds penalties scores.
    /// There is no match penalty: matches do not change the score.
    /// The penalty for any gap is length * extd_pen + open_pen. The extension pen is also applied
    /// when a gap is opened.
    #[derive(Debug, PartialEq, Eq)]
    pub struct Penalties {
        pub mismatch_pen: i32,
        pub open_pen: i32,
        pub extd_pen: i32,
    }

    /// Returned by every alignment function.
    /// The aligned strings have '-' at gaps.
    pub struct Alignment {
        pub score: i32,
        pub query_aligned: String,
        pub text_aligned: String,
    }

    /// Error type, for alignment errors.
    #[derive(Debug)]
    pub enum AlignError {
        QueryTooLong(String),
    }

    pub enum AlignResult {
        Res(Alignment),
        Error(AlignError)
    }

    /// Alignment layers. Used for tracking back.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum AlignmentLayer {
        Matches,
        Inserts,
        Deletes,
    }

    /// Main struct, implementing the algorithm.
    #[derive(Debug, PartialEq, Eq)]
    struct WavefrontState<'a> {
        query: &'a str,
        text:  &'a str,
        pens:  &'a Penalties,
        q_chars: Vec<char>,
        t_chars: Vec<char>,

        /// Counter for looping and later backtracking.
        current_score: i32,

        /// Holds the (minimal, maximal) diagonal of each iteration.
        /// Range in inclusive.
        diag_range: Vec<(i32, i32)>,

        /// Number of diagonals in the query-text alignment
        /// == to q_chars + t_chars - 1.
        num_diags: i32,

        /// The only diagonal on which we can align every char of query and
        /// text.
        final_diagonal: i32,

        /// Highest and lowest possible diags.
        highest_diag: i32,
        lowest_diag:  i32,

        /// Will store the furthest-reaching point.
        /// First index = score of that furthest-reaching point .
        /// Second      = diagonal.
        /// (just like in the article)
        matches: Vec<Vec<Option<(i32, AlignmentLayer)>>>,
        deletes: Vec<Vec<Option<(i32, AlignmentLayer)>>>,
        inserts: Vec<Vec<Option<(i32, AlignmentLayer)>>>,
    }

    /// Initializes a WavefrontState with the correct fields, for 2 string
    /// slices and a penalties struct.
    fn new_wavefront_state<'a>(query: &'a str,
                               text:  &'a str,
                               pens:  &'a Penalties) -> WavefrontState<'a> {
        let q_chars: Vec<char> = query.chars().collect();
        let t_chars: Vec<char> = text.chars().collect();

        let final_diagonal = (q_chars.len() as i32) - (t_chars.len() as i32);
        let num_diags = (q_chars.len() + t_chars.len() - 1) as i32;
        let highest_diag = q_chars.len() as i32 - 1;
        let lowest_diag = (0 - t_chars.len() as i32) + 1;

        let mut matches = vec![vec![None; num_diags as usize]; 1];
        matches[0][(0 - lowest_diag) as usize] = Some( (0, AlignmentLayer::Matches) );

        WavefrontState {
            query,
            text,
            pens,
            q_chars,
            t_chars,
            current_score: 0,
            diag_range: vec![(0, 0)],
            num_diags,
            final_diagonal,
            highest_diag,
            lowest_diag,
            matches,
            deletes: Vec::new(),
            inserts: Vec::new(),
        }
    }

    impl WavefrontState<'_> {
        fn extend(&mut self) -> () {
            //! Extends the matches wavefronts to the furthest reaching point
            //! of the current score.
            let lowest_diag  = self.diag_range[self.current_score as usize].0;
            let highest_diag = self.diag_range[self.current_score as usize].1;

            for diag in lowest_diag..=highest_diag {
                let mut query_pos = match self.at(AlignmentLayer::Matches, self.current_score, diag) {
                    Some( (val, _) ) => val + diag,
                    _                => continue,
                };
                let mut text_pos = query_pos - diag;
                // The furthest reaching point value stored is the number
                // of matched chars in the Text string.
                // For any diagonal on the dynamic programming alignment 
                // matrix, the number of chars matched for the Query is the
                // number of Text chars matched + diagonal.

                while query_pos < self.q_chars.len() as i32 && text_pos < self.t_chars.len() as i32 {
                    match (
                        self.q_chars.get(query_pos as usize),
                        self.t_chars.get(text_pos as usize),
                    ) {
                        (Some(q), Some(t)) => {
                            if q == t {
                                self.increment(diag);
                                query_pos += 1;
                                text_pos  += 1;
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
            }
        }

        fn increment_score(&mut self) -> () {
        //! Increments the current score by 1.
            self.current_score += 1;
        }

        fn increment(&mut self, diagonal: i32) -> () {
        //! Called by wf_next to increment the furthest-reaching point by 1.
            match self.at(AlignmentLayer::Matches, self.current_score, diagonal) {
                Some( (score, layer) ) => {
                    self.matches[self.current_score as usize][(diagonal - self.lowest_diag) as usize] =
                        Some( (score + 1, layer) )
                },
                None                   =>
                    self.matches[self.current_score as usize][(diagonal - self.lowest_diag) as usize] =
                        Some( (1, AlignmentLayer::Matches) )
            };
        }
        
        fn is_finished(&self) -> bool {
        //! Checks if the alignment is completed: for the current score,
        //! on the final diagonal, the furthest reaching point matches every
        //! char of Text and Query.
            match self.at(AlignmentLayer::Matches, self.current_score, self.final_diagonal) {
                Some( (score, _) ) => { 
                    if score >= self.t_chars.len() as i32 {
                        true
                    } else {
                        false
                    }
                },
                _ => false,
            }

        }


                    // WE ARE HERE
        fn next(&mut self) -> () {
        //! Equivalent of WAVEFRONT_NEXT
        
            let hi = max(1 + vec![self.diag_range.get( (self.current_score - self.pens.mismatch_pen) as usize),
                              self.diag_range.get( (self.current_score - self.pens.open_pen - self.pens.extd_pen) as usize),
                              self.diag_range.get( (self.current_score - self.pens.extd_pen) as usize)]
                             .iter()
                             .map(|x| x.unwrap_or(&(0, 0)).1 )
                             .max()
                             .unwrap(),
                         self.highest_diag);
        
            let lo = min(vec![self.diag_range.get( (self.current_score - self.pens.mismatch_pen) as usize),
                              self.diag_range.get( (self.current_score - self.pens.open_pen - self.pens.extd_pen) as usize),
                              self.diag_range.get( (self.current_score - self.pens.extd_pen) as usize)]
                             .iter()
                             .map(|x| x.unwrap_or(&(0, 0)).0 )
                             .min()
                             .unwrap() - 1,
                         self.lowest_diag);
            
            self.diag_range.push( (lo, hi) );

            self.matches.push( vec![None; (self.highest_diag - self.lowest_diag) as usize] );
            self.inserts.push( vec![None; (self.highest_diag - self.lowest_diag) as usize] );
            self.deletes.push( vec![None; (self.highest_diag - self.lowest_diag) as usize] );


            for diag in lo..=hi {
                self.update_ins(diag);
                self.update_del(diag);
                self.update_mat(diag);
            }
        }

        fn update_ins(&mut self, diag: i32) -> () {
            match (
                self.at(AlignmentLayer::Matches, self.current_score - self.pens.open_pen - self.pens.extd_pen, diag - 1),
                self.at(AlignmentLayer::Inserts, self.current_score - self.pens.extd_pen, diag - 1)
                ) {
                (None,    None)    => (),
                (Some(x), None)    => self.inserts[self.current_score as usize][(diag - self.lowest_diag) as usize] = Some((x.0, AlignmentLayer::Matches)),
                (None,    Some(x)) => self.inserts[self.current_score as usize][(diag - self.lowest_diag) as usize] = Some((x.0, AlignmentLayer::Inserts)),
                (Some(x), Some(y)) => if x.0 > y.0 {
                    self.inserts[self.current_score as usize][(diag - self.lowest_diag) as usize] = Some((x.0, AlignmentLayer::Matches));
                } else {
                    self.inserts[self.current_score as usize][(diag - self.lowest_diag) as usize] = Some((y.0, AlignmentLayer::Inserts));
                },
            }
        }

        fn update_del(&mut self, diag: i32) -> () {
            match (
                self.at(AlignmentLayer::Matches, self.current_score - self.pens.open_pen - self.pens.extd_pen, diag + 1),
                self.at(AlignmentLayer::Deletes, self.current_score - self.pens.extd_pen, diag + 1)
                ) {
                (None,    None)    => (),
                (Some(x), None)    => self.deletes[self.current_score as usize][(diag - self.lowest_diag) as usize]= Some( (x.0 + 1, AlignmentLayer::Matches) ),
                (None,    Some(x)) => self.deletes[self.current_score as usize][(diag - self.lowest_diag) as usize] = Some( (x.0 + 1, AlignmentLayer::Deletes) ),
                (Some(x), Some(y)) => if x.0 > y.0 {
                    self.deletes[self.current_score as usize][(diag - self.lowest_diag) as usize] = Some( (x.0 + 1, AlignmentLayer::Matches) );
                } else {
                    self.deletes[self.current_score as usize][(diag - self.lowest_diag) as usize] = Some( (y.0 + 1, AlignmentLayer::Deletes) );
                }
            }
        }

        fn update_mat(&mut self, diag: i32)  -> () {
            self.matches[self.current_score as usize][(diag - self.lowest_diag) as usize] = match (
                self.at(AlignmentLayer::Matches, self.current_score-self.pens.mismatch_pen, diag),
                self.at(AlignmentLayer::Inserts, self.current_score, diag),
                self.at(AlignmentLayer::Deletes, self.current_score, diag),
                ) {
                (None, None, None) => None,
                (Some(x), None, None) => Some( (x.0 + 1, AlignmentLayer::Matches) ),
                (None, Some(x), None) => Some( (x.0, AlignmentLayer::Inserts) ),
                (None, None, Some(x)) => Some( (x.0, AlignmentLayer::Deletes) ),
                (Some(x), Some(y), None) => Some( if x.0 + 1 > y.0 { (x.0 + 1, AlignmentLayer::Matches) } else { (y.0, AlignmentLayer::Inserts) } ),
                (Some(x), None, Some(y)) => Some( if x.0 + 1 > y.0 { (x.0 + 1, AlignmentLayer::Matches) } else { (y.0, AlignmentLayer::Deletes) } ),
                (None, Some(x), Some(y)) => Some( if x.0 > y.0 { (x.0, AlignmentLayer::Inserts) } else { (y.0, AlignmentLayer::Deletes) } ),
                (Some(x), Some(y), Some(z)) => Some( if x.0 + 1 > y.0 {
                                                         if x.0 + 1 > z.0 { (x.0 + 1, AlignmentLayer::Matches) }
                                                         else { (y.0, AlignmentLayer::Inserts) }
                                                    } else { 
                                                        if y.0 > z.0 {
                                                            (y.0, AlignmentLayer::Inserts) }
                                                        else { (z.0, AlignmentLayer::Deletes) }
                                                    }), 
            };
        }

        fn backtrace(&self) -> AlignResult {
            let mut curr_score = self.current_score;
            let mut curr_diag  = self.final_diagonal;
            let mut curr_layer = AlignmentLayer::Matches;

            let mut query_aligned = String::new();
            let mut text_aligned = String::new();

            while curr_score > 0 {
                match &mut curr_layer {
                    &mut AlignmentLayer::Matches => {
                        if let Some( (score, direction) ) =
                            self.matches[curr_score as usize][(curr_diag - self.lowest_diag) as usize] {
                            if let AlignmentLayer::Inserts = direction {
                                curr_layer = AlignmentLayer::Inserts;
                                continue;
                            }
                            if let AlignmentLayer::Deletes = direction {
                                curr_layer = AlignmentLayer::Deletes;
                                continue;
                            }
                            
                            let mut current_char = score;
                            curr_score -= self.pens.mismatch_pen;
                            while current_char > self.matches[curr_score as usize][(curr_diag - self.lowest_diag) as usize].unwrap().0 {
                                    query_aligned.push(self.q_chars[(current_char + curr_diag - 1) as usize]);
                                    text_aligned.push(self.t_chars[(current_char - 1) as usize]);
                                    current_char -= 1;
                            }
                        }
                    },
                    &mut AlignmentLayer::Inserts => {
                        let (chars_at_this_cell, from) =
                            match self.inserts[curr_score as usize][(curr_diag - self.lowest_diag) as usize] {
                                Some(x) => x,
                                _ => panic!(),
                            };

                        if let AlignmentLayer::Matches = from {
                            let previous = self.matches[(curr_score - self.pens.extd_pen - self.pens.open_pen) as usize][(curr_diag - self.diag_range[self.current_score as usize].0 + 1) as usize].unwrap();
                            for i in (previous.0..=chars_at_this_cell).rev() {
                                query_aligned.push(self.q_chars[(i + curr_diag - 1) as usize]);
                                text_aligned.push(self.t_chars[(i - 1) as usize]);
                            }
                            query_aligned.push(self.q_chars[(previous.0 + curr_diag - 1) as usize]);
                            text_aligned.push('-');
                            curr_diag += 1;
                            curr_score -= self.pens.extd_pen + self.pens.open_pen;
                            curr_layer = AlignmentLayer::Matches;
                        }
                        if let AlignmentLayer::Inserts = from {
                            let previous = self.inserts[(curr_score - self.pens.extd_pen) as usize][(curr_diag - self.diag_range[self.current_score as usize].0 + 1) as usize].unwrap();
                            for i in (previous.0..=chars_at_this_cell).rev() {
                                query_aligned.push(self.q_chars[(i + curr_diag - 1) as usize]);
                                text_aligned.push(self.t_chars[(i - 1) as usize]);
                            }
                            query_aligned.push(self.q_chars[(previous.0 + curr_diag - 1) as usize]);
                            text_aligned.push('-');
                            curr_diag += 1;
                            curr_score -= self.pens.extd_pen;
                        }
                    },
                    &mut AlignmentLayer::Deletes => {
                        let (chars_at_this_cell, from) =
                            match self.deletes[curr_score as usize][(curr_diag - self.diag_range[self.current_score as usize].0) as usize] {
                                Some(x) => x,
                                _ => panic!(),
                            };

                        if let AlignmentLayer::Matches = from {
                            let previous = self.matches[(curr_score - self.pens.extd_pen - self.pens.open_pen) as usize][(curr_diag - self.diag_range[self.current_score as usize].0 - 1) as usize].unwrap();
                            for i in (previous.0..chars_at_this_cell).rev() {
                                query_aligned.push(self.q_chars[(i + curr_diag - 1) as usize]);
                                text_aligned.push(self.t_chars[(i - 1) as usize]);
                            }
                            query_aligned.push('-');
                            text_aligned.push(self.t_chars[(previous.0 - 1) as usize]);
                            curr_diag -= 1;
                            curr_score -= self.pens.extd_pen + self.pens.open_pen;
                            curr_layer = AlignmentLayer::Matches;
                        }
                        if let AlignmentLayer::Deletes = from {
                            let previous = self.deletes[(curr_score - self.pens.extd_pen) as usize][(curr_diag - self.diag_range[self.current_score as usize].0 - 1) as usize].unwrap();
                            for i in (previous.0..chars_at_this_cell).rev() {
                                query_aligned.push(self.q_chars[(i + curr_diag - 1) as usize]);
                                text_aligned.push(self.t_chars[(i - 1) as usize]);
                            }
                            query_aligned.push('-');
                            text_aligned.push(self.t_chars[(previous.0 - 1) as usize]);
                            curr_diag += 1;
                            curr_score -= self.pens.extd_pen;
                        }
                    }
                }
            }
            if let AlignmentLayer::Matches = curr_layer {
                if curr_score == 0 {
                    let remaining = self.matches[0][0].unwrap().0 as usize;
                    if remaining > 0 {
                           query_aligned = query_aligned + &self.q_chars[..remaining].iter()
                                                                                     .rev()
                                                                                     .collect::<String>();
                           text_aligned  = text_aligned + &self.t_chars[..remaining].iter()
                                                                                    .rev()
                                                                                    .collect::<String>();
                    }
                }
            }

            let q = query_aligned.chars().rev().collect();
            let t = text_aligned.chars().rev().collect();

            AlignResult::Res(Alignment {
                score: self.current_score,
                query_aligned: q,
                text_aligned: t,
                })
            }


        fn at(&self, layer: AlignmentLayer, score: i32, diag: i32) -> Option<(i32, AlignmentLayer)> {
                    if score < 0 {
                        return None;
                    }
                    if score > self.current_score {
                        return None;
                    }
                    if diag > self.diag_range[score as usize].1 || diag < self.diag_range[score as usize].0 {
                        return None;
                    }
                    match layer {
                        AlignmentLayer::Matches => self.matches[score as usize][(diag - self.lowest_diag) as usize].clone(),
                        AlignmentLayer::Inserts => self.inserts[score as usize][(diag - self.lowest_diag) as usize].clone(),
                        AlignmentLayer::Deletes => self.deletes[score as usize][(diag - self.lowest_diag) as usize].clone(),

                    }
            }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_new_wavefront_state() -> () {
        // Doesn't do much currently but at least if we accidently
        // change the behaviour/meaning of the wavefront state structs,
        // we'll notice.
            let state = new_wavefront_state("GATA",
                "TAGAC",
                &Penalties {
                    mismatch_pen: 1,
                    open_pen: 2,
                    extd_pen: 3,
                }
            );
            
            let mut manual_matches = vec![vec![None; 8]; 1];
            manual_matches[0][4] = Some( (0, AlignmentLayer::Matches) );
            let manual = WavefrontState {
                query: "GATA",
                text:  "TAGAC",
                pens: &Penalties {
                    mismatch_pen: 1,
                    open_pen: 2,
                    extd_pen: 3,
                },
                q_chars: "GATA".chars().collect(),
                t_chars: "TAGAC".chars().collect(),
                current_score: 0,
                diag_range: vec![(0, 0)],
                num_diags: 8,
                final_diagonal: -1,
                highest_diag: 3,
                lowest_diag: -4,
                matches: manual_matches,
                deletes: Vec::new(),
                inserts: Vec::new(),
            };

            assert_eq!(state, manual);
        }

        #[test]
        fn test_wavefront_at() -> () {
            let mut wf = new_wavefront_state("helo", "hello", &Penalties{
                mismatch_pen: 1,
                open_pen: 1,
                extd_pen: 1,
            });
            assert_eq!(wf.at(AlignmentLayer::Matches, 0, 0),
                Some( (0, AlignmentLayer::Matches) )
            );
            assert_eq!(wf.at(AlignmentLayer::Matches, 0, -4),
                None
            );

            wf.matches[0][4] = Some( (10, AlignmentLayer::Inserts) );
            assert_eq!(wf.at(AlignmentLayer::Matches, 0, 0),
                Some( (10, AlignmentLayer::Inserts) )
            );

            wf.matches[0][4] = None;
            assert_eq!(wf.at(AlignmentLayer::Matches, 0, -4),
                None
            );

            wf.matches[0][7] = Some( (-10, AlignmentLayer::Matches) );
            assert_eq!(wf.at(AlignmentLayer::Matches, 0, 3),
                None // out of the diag range
            );

            wf.diag_range[0] = (-4, 3);
            assert_eq!(wf.at(AlignmentLayer::Matches, 0, 3),
                Some( (-10, AlignmentLayer::Matches) )
            );

            assert_eq!(wf.at(AlignmentLayer::Matches, 0, -4),
                None
            );
            wf.matches[0][0] = Some( (-100, AlignmentLayer::Matches) );
            assert_eq!(wf.at(AlignmentLayer::Matches, 0, -4),
                Some( (-100, AlignmentLayer::Matches) )
            );
}

        #[test]
        fn test_wavefront_extend_match() -> () {
            let mut wf = new_wavefront_state("ATAC", "ATACA", &Penalties {
                mismatch_pen: 1,
                open_pen: 1,
                extd_pen: 1,
            });
            wf.extend();
            assert_eq!(wf.matches[0][4], Some( (4, AlignmentLayer::Matches) ) );
        }
        #[test]
        fn test_wavefront_extend_mismatch() -> () {
            let mut wf = new_wavefront_state("ZZZ", "TACA", &Penalties {
                mismatch_pen: 1,
                open_pen: 1,
                extd_pen: 1,
            });
            wf.extend();
            assert_eq!(wf.matches[0][3], Some( (0, AlignmentLayer::Matches) ) );
        }

        #[test]
        fn test_wavefront_increment_score() -> () {
            let mut wf = new_wavefront_state("ZZZZ", "ATACA", &Penalties {
                mismatch_pen: 1,
                open_pen: 1,
                extd_pen: 1,
            });
            assert_eq!(wf.current_score, 0);
            wf.increment_score();
            wf.increment_score();
            assert_eq!(wf.current_score, 2);
        }

        #[test]
        fn test_wavefront_increment() -> () {
            let mut wf = new_wavefront_state("ZZZZZ", "CATACA", &Penalties {
                    mismatch_pen: 1,
                    open_pen: 1,
                    extd_pen: 1,
                });
            assert_eq!(wf.matches[0][5], Some( (0, AlignmentLayer::Matches) ) ); 
            wf.increment(0);
            wf.increment(0);
            assert_eq!(wf.matches[0][5], Some( (2, AlignmentLayer::Matches) ) ); 
        }

        #[test]
        fn test_wavefront_is_finished() -> () {
            let mut wf = new_wavefront_state("AAAA", "AAAA", &Penalties {
                            mismatch_pen: 1,
                            open_pen: 1,
                            extd_pen: 1,
                        });
            assert!(!wf.is_finished());
            wf.extend();
            assert!(wf.is_finished());

            let mut wf = new_wavefront_state("AAAA", "AAAAT", &Penalties {
                            mismatch_pen: 1,
                            open_pen: 1,
                            extd_pen: 1,
                        });
            assert!(!wf.is_finished());
            wf.extend();
            assert!(!wf.is_finished());
        }
    }
}
