/// Holds penalties scores.
/// There is no match penalty: matches do not change the score.
/// The penalty for any gap is length * extd_pen + open_pen. The extension pen is also applied
/// when a gap is opened.
/// Penalties should be a positive int.
#[derive(Debug, PartialEq, Eq)]
pub struct Penalties {
    pub mismatch_pen: i32,
    pub open_pen: i32,
    pub extd_pen: i32,
}

/// Returned by every alignment function.
/// The aligned strings have '-' at gaps.
#[derive(Debug, Eq, PartialEq)]
pub struct Alignment {
    pub score: i32,
    pub query_aligned: String,
    pub text_aligned: String,
}

/// Error type, for alignment errors.
#[derive(Debug, Eq, PartialEq)]
pub enum AlignError {
    ZeroLength(String),
    QueryTooLong(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AlignResult {
    Res(Alignment),
    Error(AlignError)
}

/// Alignment layers. Used for tracking back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentLayer {
    Matches,
    Inserts,
    Deletes,
}
