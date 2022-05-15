//! General types and functions that are useful for alignments.

/// Holds penalties scores.
/// There is no match penalty: matches do not change the score.
/// The penalty for any gap is length * extd_pen + open_pen. The extension pen is also applied
/// when a gap is opened.
/// Penalties should be a positive int.
use strum_macros::{Display, EnumString};

/// The different alignment algorithms implemented in this crate.
#[derive(Clone, Copy, Debug, EnumString, Display)]
pub enum AlignmentAlgorithm {
    /// Basic WFA.
    Wavefront,
    
    WavefrontAdaptive,

    /// DP matrix based, gap-affine, unoptimized alignment.
    SWG,
}

/// Penalties used for WFA.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Penalties {
    /// There is a single mismatch penalty for every char combination.
    /// WFA requires that the match penalty is set to 0.
    pub mismatch_pen: u32,

    /// Gap opening penalty.
    pub open_pen: u32,

    /// Gap extension penalty. It is also applied when a gap is opened.
    pub extd_pen: u32,
}

/// This is the value returned by every alignment function after successfully aligning 2 strings.
/// The aligned strings have '-' at gaps.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Alignment {
    pub score: u32,
    pub query_aligned: String,
    pub text_aligned: String,
}

/// Error type, for alignment errors.
#[derive(Debug, Eq, PartialEq)]
pub enum AlignmentError {
    /// Both strings should have at least 1 character.
    ZeroLength(String),

    /// query.len() needs to be <= to text.len()
    QueryTooLong(String),
}

/// Alignment layers. Used for tracking back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentLayer {
    Matches,
    Inserts,
    Deletes,
}

/// The methods for every wavefront type.
pub(crate) trait Wavefront {
    fn extend(&mut self);
    fn next(&mut self);
    fn increment_score(&mut self);
    fn is_finished(&self) -> bool;
    fn backtrace(&self) -> Result<Alignment, AlignmentError>;
}

/// Used to store and access wavefronts efficiently.
/// T is the type used to store the number of chars matched.
/// U is the type used for diagonals.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WavefrontGrid {
    /// The vec of (lowest valid diag, highest valid diag) for each score.
    /// Lowest is always a negative value, stored using an unsigned type.
    diags: Vec<(i32, i32)>,

    /// The vec that stores the offset at which each layer starts in the vector.
    /// Each layer corresponds to a score.
    offsets: Vec<usize>,

    matches: Vec<Option<(u32, AlignmentLayer)>>,
    inserts: Vec<Option<(u32, AlignmentLayer)>>,
    deletes: Vec<Option<(u32, AlignmentLayer)>>,
}

/// Make a new wavefront grid with the first diagonal of (lo, hi)
/// lo and hi = 0 for a 1-element initial diagonal.
pub(crate) fn new_wavefront_grid() -> WavefrontGrid {
    let diags = vec![(0, 0)];
    // Stores the tuple of the (lowest, highest) diagonals for a given score.
    // Initial value = (0, 0) => the last value is included.
    // The first tuple item stores the lowest diagonal, and stores values <= 0.

    let matches = vec![Some((0, AlignmentLayer::Matches)); 1];
    let inserts = vec![None; 1];
    let deletes = vec![None; 1];

    let offsets = vec![0, 1];
    // The furthest-reaching point will be stored in the previous 3 vecs.
    // These vecs are 1D: instead of indicing them by 2D Vecs of v[score][diagonal],
    // we'll indice them as:
    //      v[offsets[score] + (diagonal - lowest_diag_at_that_score)]
    //
    // Thus, offsets stores the index at which a given score starts in the 3 previous vecs.
    //
    // Whenever we add a layer, we'll push n None values in the 3 vecs,
    // with None = highest_diag - lowest_diag + 1
    //      => We'll know in advance at which offset will the next score start.
    //      Therefore, offsets' last value will always be in advance by 1.

    WavefrontGrid {
        diags,
        offsets,
        matches,
        inserts,
        deletes,
    }
}

impl WavefrontGrid {
    /// Add a new layer to the wavefronts.
    /// lo and hi are the lowest/highest diagonals for this new layer.
    pub(crate) fn add_layer(&mut self, lo: i32, hi: i32) {
        self.diags.push((lo, hi));

        let new_width: usize = (hi - lo + 1) as usize;
        self.offsets
            .push(self.offsets[self.offsets.len() - 1] + new_width);

        for _ in lo..=hi {
            self.matches.push(None);
            self.inserts.push(None);
            self.deletes.push(None);
        }
    }

    /// Get a value.
    pub(crate) fn get(
        &self,
        layer: AlignmentLayer,
        score: u32,
        diag: i32,
    ) -> Option<(u32, AlignmentLayer)> {
        let score = score as usize;
        if score >= self.offsets.len() || diag < self.diags[score].0 || diag > self.diags[score].1 {
            // offsets is always ahead by 1, since we know the len of a layer
            // when it's created. Adding a new layer updates the offset of the next layer.
            None
        } else {
            let diag_offset = (diag - self.diags[score].0) as usize;
            let position: usize = self.offsets[score] + diag_offset;
            match layer {
                AlignmentLayer::Matches => self.matches[position],
                AlignmentLayer::Inserts => self.inserts[position],
                AlignmentLayer::Deletes => self.deletes[position],
            }
        }
    }

    pub(crate) fn set(
        &mut self,
        layer: AlignmentLayer,
        score: u32,
        diag: i32,
        value: Option<(u32, AlignmentLayer)>,
    ) {
        let score = score as usize;
        if score < self.offsets.len() && diag >= self.diags[score].0 && diag <= self.diags[score].1
        {
            let position = self.offsets[score] + (diag - self.diags[score].0) as usize;
            match layer {
                AlignmentLayer::Matches => self.matches[position] = value,
                AlignmentLayer::Inserts => self.inserts[position] = value,
                AlignmentLayer::Deletes => self.deletes[position] = value,
            };
        }
    }

    pub(crate) fn get_diag_range(&self, score: u32) -> Option<&(i32, i32)> {
        self.diags.get(score as usize)
    }

    pub(crate) fn increment(&mut self, score: u32, diag: i32) {
        let score = score as usize;
        let position = self.offsets[score] + (diag - self.diags[score].0) as usize;
        self.matches[position] = match self.matches[position] {
            Some((score, direction)) => Some((score + 1, direction)),
            None => Some((1, AlignmentLayer::Matches)),
        };
    }
}

#[cfg(test)]
mod tests_wfgrid {
    use super::*;

    #[test]
    fn test_new_wfgrid() {
        let grid: WavefrontGrid = new_wavefront_grid();
        assert_eq!(grid.diags[0], (0, 0));
        assert_eq!(grid.offsets[0], 0);
        assert_eq!(grid.offsets[1], 1);
        assert_eq!(grid.matches[0], Some((0, AlignmentLayer::Matches)));
        assert_eq!(grid.inserts[0], None);
        assert_eq!(grid.deletes[0], None);
    }

    #[test]
    fn test_add_layer() {
        let mut grid: WavefrontGrid = new_wavefront_grid();
        grid.add_layer(-3, 3);
        assert_eq!(grid.diags[0], (0, 0));
        assert_eq!(grid.diags[1], (-3, 3));
        assert_eq!(grid.offsets[0], 0);
        assert_eq!(grid.offsets[1], 1);
        assert_eq!(grid.offsets[2], 8);
        assert_eq!(grid.matches.len(), 8); // initial = 0, next cycle has 7 values
        assert_eq!(grid.inserts.len(), 8);
        assert_eq!(grid.deletes.len(), 8);
    }
}
