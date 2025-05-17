use chess::{Board, Piece};

use crate::score::Score;

pub fn eval_heuristic(board: &Board) -> Score {
    material_eval(board)
}

/// Scores the provided board based on pure material value, assuming that we are up to move
fn material_eval(board: &Board) -> Score {
    let mine = board.color_combined(board.side_to_move());
    let theirs = board.color_combined(!board.side_to_move());

    // Get pieces and do sums
    let mut cp: i16 = 0;

    cp += ((board.pieces(Piece::Pawn) & *mine).0.count_ones() * 100) as i16;
    cp -= ((board.pieces(Piece::Pawn) & *theirs).0.count_ones() * 100) as i16;

    cp += ((board.pieces(Piece::Knight) & *mine).0.count_ones() * 350) as i16;
    cp -= ((board.pieces(Piece::Knight) & *theirs).0.count_ones() * 350) as i16;

    cp += ((board.pieces(Piece::Bishop) & *mine).0.count_ones() * 350) as i16;
    cp -= ((board.pieces(Piece::Bishop) & *theirs).0.count_ones() * 350) as i16;

    cp += ((board.pieces(Piece::Rook) & *mine).0.count_ones() * 525) as i16;
    cp -= ((board.pieces(Piece::Rook) & *theirs).0.count_ones() * 525) as i16;

    cp += ((board.pieces(Piece::Queen) & *mine).0.count_ones() * 1000) as i16;
    cp -= ((board.pieces(Piece::Queen) & *theirs).0.count_ones() * 1000) as i16;

    Score::cp(cp)
}
