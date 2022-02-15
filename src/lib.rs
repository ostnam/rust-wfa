pub mod wavefront {
    pub struct Penalties {
        pub match_pen: i32,
        pub mismatch_pen: i32,
        pub open_pen: i32,
        pub extd_pen: i32,
    }

    pub struct Alignment {
        pub score: i32,
        pub query_aligned: String,
        pub text_aligned: String,
    }

    #[derive(Debug)]
    pub enum AlignError {
        QueryTooLong(String),
    }

    enum AlignmentLayer {
        Matches,
        Inserts,
        Deletes,
    }

    struct WavefrontVec {
        kind: AlignmentLayer,
        diag_range: (i32, i32),
        values: Vec<Vec<Option<i32>>>,
    }

    impl WavefrontVec {
        fn add_wave(&mut self) {
            self.values
                .push( vec![None;   
                    (self.diag_range.1 - self.diag_range.0) as usize
                    ]
                 );
        }

        fn at(&self, score: i32, diag: i32) -> Option<i32> {
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
    }

    pub fn wavefront_align(
        query: &str,
        text: &str,
        pens: &Penalties,
    ) -> Result<Alignment, AlignError> {
        if query.len() > text.len() {
            return Err(AlignError::QueryTooLong("Query is longer than the reference string. The length of the first string must be <= to the the length of the second string".to_string()));
        }
        let final_diagonal: i32 = query.len() as i32 - text.len() as i32;

        let q_chars: Vec<char> = query.to_string().chars().collect();
        let t_chars: Vec<char> = text.to_string().chars().collect();

        let mut matches_front = WavefrontVec {
            kind: AlignmentLayer::Matches,
            diag_range: (0 - text.len() as i32, 0 + query.len() as i32),
            values: Vec::new(),
        };


        let mut inserts_front = WavefrontVec {
            kind: AlignmentLayer::Inserts,
            diag_range: (0 - text.len() as i32, 0 + query.len() as i32),
            values: Vec::new(),
        };

        let mut deletes_front = WavefrontVec {
            kind: AlignmentLayer::Deletes,
            diag_range: (0 - text.len() as i32, 0 + query.len() as i32),
            values: Vec::new(),
        };

        let mut current_score: i32 = 0;
        let mut lowest_diag: i32 = 0;
        let mut highest_diag: i32 = 0;

        matches_front.add_wave();
        matches_front.values[0][(0 - matches_front.diag_range.0) as usize] = Some(0);
        inserts_front.add_wave();
        deletes_front.add_wave();

        loop {
            wavefront_extend(
                &mut matches_front,
                &q_chars,
                &t_chars,
                current_score,
                lowest_diag,
                highest_diag,
            );

            match matches_front.at(current_score, final_diagonal) {
                Some(a) => {
                    if a == text.len() as i32 {
                        break;
                    }
                }
                _ => (),
            }

            current_score += 1;
            matches_front.add_wave();
            inserts_front.add_wave();
            deletes_front.add_wave();
            wavefront_next(
                &mut matches_front,
                &mut inserts_front,
                &mut deletes_front,
                &current_score,
                &mut lowest_diag,
                &mut highest_diag,
                &pens
            );
        }
        wavefront_backtrace(
            &matches_front,
            &inserts_front,
            &deletes_front,
            &q_chars,
            &t_chars,
            current_score,
            final_diagonal,
            pens,
        )
    }

    fn wavefront_extend(
        front: &mut WavefrontVec,
        q_chars: &Vec<char>,
        t_chars: &Vec<char>,
        current_score: i32,
        lowest_diag: i32,
        highest_diag: i32,
    ) -> () {
        for diag in lowest_diag..highest_diag + 1 {
            let mut query_pos = match front.at(current_score, diag) {
                Some(a) => a + diag,
                _       => continue,
            };
            let mut text_pos = match front.at(current_score, diag) {
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
                            front.increment(current_score, diag);
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
