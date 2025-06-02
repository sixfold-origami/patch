use std::collections::HashMap;

use chess::{Board, ChessMove};

use crate::score::Score;

/// A [transposition table](https://www.chessprogramming.org/Transposition_Table),
/// storing computed scores and node types for each [`Board`] visited
pub type TranspositionTable = HashMap<Board, TranspositionData>;

/// The bundle of data for a single position in the transposition table
#[derive(Debug, Clone)]
pub struct TranspositionData {
    /// The score that this board recieved
    pub score: Score,
    /// The type of node this is in the tree search
    ///
    /// Node type determines [how the score is interpreted](https://www.chessprogramming.org/Transposition_Table#What_Information_is_Stored)
    pub ty: NodeType,
    /// Either the best move or refutation move found for this position
    pub mv: ChessMove,
    /// The depth that this evaluation came from
    pub depth: u8,
}

impl TranspositionData {
    /// Constructs a new `Self`
    pub fn new(score: Score, ty: NodeType, mv: ChessMove, depth: u8) -> Self {
        Self {
            score,
            ty,
            mv,
            depth,
        }
    }
}

/// The node type evaluated in the transposition table
///
/// See: https://www.chessprogramming.org/Node_Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Pv,
    Cut,
    All,
}
