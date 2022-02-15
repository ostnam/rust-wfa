/// This module defines all the types and functions used in the crate.
pub mod wavefront {

    /// This function is exported and can be called to perform an alignment.
    /// The query cannot be longer than the text.
    pub fn wavefront_align(query: &str, text: &str, pens: &Penalties) 
        -> Result<Alignment, AlignError> {
        if query.len() > text.len() {
            return Err(
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

            current_front.increment();

            current_front.next();
        }
        current_front.backtrace()
    }

    /// This function is exported and can be called to perform an alignment.
    /// The query cannot be longer than the text.
    pub fn wavefront_align_adaptive(query: &str,
                                    text: &str,
                                    pens: &Penalties) 
        -> Result<Alignment, AlignError> {
        if query.len() > text.len() {
            return Err(
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
            current_front.increment();             // Add 1 to the score.
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

    /// Alignment layers. Used for tracking back.
    #[derive(Debug, Clone, PartialEq, Eq)]
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
        current_score: i32,
        diag_range: Vec<(i32, i32)>,
        num_diags: i32,
        final_diagonal: i32,
        matches: Vec<Vec<Option<(i32, AlignmentLayer)>>>,
        deletes: Vec<Vec<Option<(i32, AlignmentLayer)>>>,
        inserts: Vec<Vec<Option<(i32, AlignmentLayer)>>>,
    }

    fn new_wavefront_state<'a>(query: &'a str,
                               text:  &'a str,
                               pens:  &'a Penalties) -> WavefrontState<'a> {
        let q_chars: Vec<char> = query.chars().collect();
        let t_chars: Vec<char> = text.chars().collect();

        let final_diagonal = (q_chars.len() as i32) - (t_chars.len() as i32);
        let num_diags = (q_chars.len() + t_chars.len() - 1) as i32;

        let mut matches = vec![vec![None; num_diags as usize]; 1];
        WavefrontState {
            query,
            text,
            pens,
            q_chars,
            t_chars,
            current_score: 0,
            diag_range: vec![(0, 1)],
            num_diags,
            final_diagonal,
            matches,
            deletes: Vec::new(),
            inserts: Vec::new(),
        }
    }

                                // WE ARE HERE
    impl WavefrontState<'_> {
        fn wavefront_extend(&mut self) -> () {
            let lowest_diag  = self.diag_range[self.current_score as usize].0;
            let highest_diag = self.diag_range[self.current_score as usize].1;

            for diag in lowest_diag..=highest_diag {
                let mut query_pos = match self.matches.at(self.current_score, diag) {
                    Some(a) => a + diag,
                    _       => continue,
                };
                let mut text_pos = match self.matches.at(self.current_score, diag) {
                    Some(a) => a,
                    _       => continue,
                };

                while query_pos < q_chars.len() as i32 && text_pos < t_chars.len() as i32 {
                    match (
                        q_chars.get(query_pos as usize),
                        t_chars.get(text_pos as usize),
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
        fn increment(self, diagonal: i32) -> () {
            self.matches[curr_score][diagonal] += 1;
        }


    fn wavefront_next(
        matches: &mut WavefrontVec,
        inserts: &mut WavefrontVec,
        deletes: &mut WavefrontVec,
        score: &i32,
        lo: &mut i32,
        hi: &mut i32,
        pens: &Penalties,
    ) -> () {
        *hi += 1;
        *lo -= 1;

        for diag in *lo..*hi + 1 {
            inserts.update_ins(score, &diag, &matches, &pens);
            deletes.update_del(score, &diag, &matches, &pens);
            matches.update_mat(score, &diag, &inserts, &deletes, &pens);
        }
    }

    fn wavefront_backtrace(
        matches: &WavefrontVec,
        inserts: &WavefrontVec,
        deletes: &WavefrontVec,
        q_chars: &Vec<char>,
        t_chars: &Vec<char>,
        score: i32,
        final_diagonal: i32,
        pens: &Penalties,
    ) -> Result<Alignment, AlignError> {

        let mut curr_score = score;
        let mut curr_layer = AlignmentLayer::Matches;
        let mut curr_diag = final_diagonal;

        let mut query_aligned = String::new();
        let mut text_aligned = String::new();

        while curr_score > 0 {
            match &mut curr_layer {
                &mut AlignmentLayer::Matches => {
                    match matches.at(curr_score, curr_diag) {
                        None    => panic!(),
                        Some(mut x) => {
                            while x + curr_diag - 1 >= 0 &&
                                  x - 1 >= 0 &&
                                  q_chars[(x + curr_diag - 1) as usize] == t_chars[(x - 1) as usize] {
                                query_aligned.push(q_chars[(x + curr_diag - 1) as usize]);
                                text_aligned.push(t_chars[(x - 1) as usize]);
                                x -= 1;
                            }

                            if let Some(y) = inserts.at(curr_score, curr_diag) {
                                if x == y {
                                  curr_layer = AlignmentLayer::Inserts;
                                  continue;
                                }
                            }

                            if let Some(y) = deletes.at(curr_score, curr_diag) {
                                if x == y + 1 {
                                  curr_layer = AlignmentLayer::Deletes;
                                  continue;
                                }
                            }
                        query_aligned.push(q_chars[(x + curr_diag - 1) as usize]);
                        text_aligned.push(t_chars[(x - 1) as usize]);
                        curr_score -= pens.mismatch_pen;
                        }
                    }
                },
                &mut AlignmentLayer::Inserts => {
                    let current   = inserts.at(curr_score, curr_diag).unwrap();
                    let from_open = matches.at(curr_score - pens.open_pen - pens.extd_pen, curr_diag - 1);
                    query_aligned.push(q_chars[(current + curr_diag - 1) as usize]);
                    text_aligned.push('-');

                    if let Some(x) = from_open {
                        if x == current {
                            curr_layer = AlignmentLayer::Matches;
                            curr_score -= pens.open_pen;
                        }
                    }

                    curr_diag -= 1;
                    curr_score -= pens.extd_pen;
                },

                &mut AlignmentLayer::Deletes => {
                    let current   = deletes.at(curr_score, curr_diag).unwrap();
                    let from_open = matches.at(curr_score - pens.open_pen - pens.extd_pen, curr_diag + 1);
                    query_aligned.push('-');
                    text_aligned.push(t_chars[(current - 1) as usize]);

                    if let Some(x) = from_open {
                        if 1 + x == current {
                            curr_layer = AlignmentLayer::Matches;
                            curr_score -= pens.open_pen;
                        }
                    }
                    curr_diag += 1;
                    curr_score -= pens.extd_pen;
                },
            }
        }

        if let AlignmentLayer::Matches = curr_layer {
            if curr_score == 0 {
                if matches.at(curr_score, 0).unwrap_or(0) > 0 {
                   query_aligned = query_aligned + &q_chars[..matches.at(0, 0).unwrap() as usize].iter()
                                                                                .rev()
                                                                                .collect::<String>();
                   text_aligned  = text_aligned + &t_chars[..matches.at(0, 0).unwrap() as usize].iter()
                                                                                .rev()
                                                                                .collect::<String>();
                }
            }
        }

        let q = query_aligned.chars().rev().collect();
        let t = text_aligned.chars().rev().collect();

        Ok(Alignment {
            score,
            query_aligned: q,
            text_aligned: t,
            })
        }
    }
    fn add_wave(&mut self, width: usize) {
                self.values
                    .push(
                        vec![None; width]
                        );
            }

            fn at(&self, score: i32, diag: i32) -> Option<(i32, AlignmentLayer)> {
                if score < 0 {
                    return None;
                }
                if score >= self.values.len() as i32 {
                    return None;
                }
                if diag >= self.diag_range.1 || diag < self.diag_range.0 {
                    return None;
                }
                self.values[score as usize][(diag - self.diag_range.0) as usize]
            }

            fn set(&mut self, score: i32, diag: i32, value: i32) -> () {
                if (score as usize) < self.values.len()
                    && score >= 0
                    && diag >= self.diag_range.0
                    && diag < self.diag_range.1
                {
                    self.values[score as usize][(diag - self.diag_range.0) as usize] = Some(value);
                }
            }

            fn increment(&mut self, score: i32, diag: i32) -> () {
                if let Some(previous) = self.at(score, diag) {
                    self.set(score, diag, previous + 1);
                }
            }

            fn update_ins( &mut self,
                           score: &i32,
                           diag: &i32,
                           matches: &WavefrontVec,
                           pens: &Penalties        ) -> () {
                match (&self.kind, &matches.kind) {
                    (AlignmentLayer::Inserts, AlignmentLayer::Matches) => (),
                    _ => panic!("update_ins called on two layers of incorrect type"),
                }
                match (
                    matches.at(score - pens.open_pen - pens.extd_pen, diag - 1),
                    self.at(score - pens.extd_pen, diag - 1)
                    ) {
                    (None,    None)    => (),
                    (Some(x), None)    => self.set(*score, *diag, x),
                    (None,    Some(x)) => self.set(*score, *diag, x),
                    (Some(x), Some(y)) => self.set(*score, *diag, if x > y {x} else {y} ),
                }
            }

            fn update_del( &mut self,
                           score: &i32,
                           diag: &i32,
                           matches: &WavefrontVec,
                           pens: &Penalties        ) -> () {
                match (&self.kind, &matches.kind) {
                    (AlignmentLayer::Deletes, AlignmentLayer::Matches) => (),
                    _ => panic!("update_ins called on two layers of incorrect type"),
                }
                match (
                    matches.at(score - pens.open_pen - pens.extd_pen, diag + 1),
                    self.at(score - pens.extd_pen, diag + 1)
                    ) {
                    (None,    None)    => (),
                    (Some(x), None)    => self.set(*score, *diag, 1 + x),
                    (None,    Some(x)) => self.set(*score, *diag, 1 + x),
                    (Some(x), Some(y)) => self.set(*score, *diag, 1 + if x > y {x} else {y} ),
                }
            }
            fn update_mat(
                &mut self,
                score: &i32,
                diag: &i32,
                inserts: &WavefrontVec,
                deletes: &WavefrontVec,
                pens: &Penalties,
            ) -> () {
                match (&self.kind, &inserts.kind, &deletes.kind) {
                    (AlignmentLayer::Matches, AlignmentLayer::Inserts, AlignmentLayer::Deletes) => (),
                    _ => panic!("update_ins called on two layers of incorrect type"),
                }
                let largest = match (
                    self.at(score-pens.mismatch_pen, *diag),
                        inserts.at(*score, *diag),
                        deletes.at(*score, *diag),
                    ) {
                    (None, None, None) => None,
                    (Some(x), None, None) => Some(x + 1),
                    (None, Some(x), None) => Some(x),
                    (None, None, Some(x)) => Some(x),
                    (Some(x), Some(y), None) => Some( if x + 1 > y { x + 1 } else { y } ),
                    (Some(x), None, Some(y)) => Some( if x + 1 > y { x + 1 } else { y } ),
                    (None, Some(x), Some(y)) => Some( if x > y { x } else { y } ),
                    (Some(x), Some(y), Some(z)) => Some( if x + 1 > y { if x + 1 > z {x + 1} else {z} } else { if y > z {y} else {z}}), 
                };
                if let Some(x) = largest {
                    self.set(*score, *diag, x);
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
                diag_range: vec![(0, 1)],
                num_diags: 6,
                final_diagonal: -1,
                matches: vec![vec![None; 6]; 1],
                deletes: Vec::new(),
                inserts: Vec::new(),
            };

            assert_eq!(state, manual);
        }
    }
}
