use std::cmp::Ordering;

use uci_parser::UciScore;

/// Score evaluation for a position
///
/// Scores are always from the perspective of the current player,
/// so postive scores are winning, and negative scores are losing
#[derive(Debug, PartialEq, Eq)]
pub enum Score {
    /// Score advantage in centipawns
    Centipawns(i16),
    /// Mate in x turns
    ///
    /// Positive is that we can mate in that many turns,
    /// negative is getting mated in that many turns.
    /// A value of zero means we are currently in checkmate.
    Mate(i16),
}

impl Score {
    /// Create [`Self`] with the provided centipawn score
    pub fn cp(score: i16) -> Self {
        Self::Centipawns(score)
    }

    /// Create [`Self`] with the provided mate score
    pub fn mate(moves: i16) -> Self {
        Self::Mate(moves)
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // If both scores are the same type, then the internal values can be compared directly
            (Score::Centipawns(s1), Score::Centipawns(s2)) => Some(s1.cmp(s2)),
            (Score::Mate(m1), Score::Mate(m2)) => Some(m1.cmp(m2)),
            // If they're different, then the ordering is always:
            // (we get mated) < (negative score) < (positive score) < (they get mated)
            (Score::Centipawns(_), Score::Mate(m)) => {
                if m <= &0 {
                    // We are getting mated.
                    // Any situation where we are not getting mated is a higher score
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Score::Mate(m), Score::Centipawns(_)) => {
                if m <= &0 {
                    // We are getting mated.
                    // Any situation where we are not getting mated is a higher score
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
        }
    }
}

impl From<Score> for UciScore {
    fn from(value: Score) -> Self {
        match value {
            Score::Centipawns(cp) => UciScore::cp(cp as i32),
            Score::Mate(m) => UciScore::mate(m as i32),
        }
    }
}
