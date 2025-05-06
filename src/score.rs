//! See [`Score`]

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
    Mate(i8),
}

impl Score {
    /// Create [`Self`] with the provided centipawn score
    pub fn cp(score: i16) -> Self {
        Self::Centipawns(score)
    }

    /// Create [`Self`] with the provided mate score
    pub fn mate(moves: i8) -> Self {
        Self::Mate(moves)
    }

    /// Semantically inverts `self`, to evaluate this score from the opponents perspective
    ///
    /// In the case of mate scores, the mate counter is increased,
    /// as this is more convenient in the minimax algorithm.
    pub fn flip(self) -> Self {
        match self {
            Score::Centipawns(cp) => Score::Centipawns(-cp),
            Score::Mate(m) => {
                if m.is_positive() {
                    Score::Mate((m.abs() + 1) * -1)
                } else {
                    Score::Mate(m.abs() + 1)
                }
            }
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // If both scores are in centipawns, then the internal values can be compared directly
            (Score::Centipawns(s1), Score::Centipawns(s2)) => Some(s1.cmp(s2)),
            // If they're different, then the ordering is always:
            // (we get mated) < (negative centipawns) < (positive centipawns) < (they get mated)
            (Score::Centipawns(_), Score::Mate(m)) => {
                if m <= &0 {
                    // We are getting mated.
                    // Any situation where we are not getting mated is a higher score
                    Some(Ordering::Greater)
                } else {
                    // They are getting mated.
                    // Any situation in which we are not mating them is a lower score
                    Some(Ordering::Less)
                }
            }
            (Score::Mate(m), Score::Centipawns(_)) => {
                if m <= &0 {
                    // We are getting mated.
                    // Any situation where we are not getting mated is a higher score
                    Some(Ordering::Less)
                } else {
                    // They are getting mated.
                    // Any situation in which we are not mating them is a lower score
                    Some(Ordering::Greater)
                }
            }
            // If both scores are mates, then the order is:
            // M0 < -M1 < -Mn < Mn < M1
            (Score::Mate(m1), Score::Mate(m2)) => {
                if m1 <= &0 && m2 <= &0 {
                    Some(m2.cmp(m1)) // Reverse, since lower values are actually better
                } else if m1 <= &0 && m2.is_positive() {
                    Some(Ordering::Less)
                } else if m1.is_positive() && m2 <= &0 {
                    Some(Ordering::Greater)
                } else {
                    Some(m2.cmp(m1)) // Again, lower is better
                }
            }
        }
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
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
