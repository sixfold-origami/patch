use chess::{Board, Piece};

use crate::score::Score;

/// Evaluation heuristic based on material and piece positions
pub fn eval_heuristic(board: &Board) -> Score {
    let score = material_eval(board);

    Score::cp(score)
}

/// Scores the provided board based on pure material value, assuming that we are up to move
fn material_eval(board: &Board) -> i16 {
    let mine = board.color_combined(board.side_to_move());
    let theirs = board.color_combined(!board.side_to_move());

    // Get pieces and do sums
    let mut cp: i16 = 0;

    cp += ((board.pieces(Piece::Pawn) & *mine).popcnt() * 100) as i16;
    cp -= ((board.pieces(Piece::Pawn) & *theirs).popcnt() * 100) as i16;

    cp += ((board.pieces(Piece::Knight) & *mine).popcnt() * 350) as i16;
    cp -= ((board.pieces(Piece::Knight) & *theirs).popcnt() * 350) as i16;

    cp += ((board.pieces(Piece::Bishop) & *mine).popcnt() * 350) as i16;
    cp -= ((board.pieces(Piece::Bishop) & *theirs).popcnt() * 350) as i16;

    cp += ((board.pieces(Piece::Rook) & *mine).popcnt() * 525) as i16;
    cp -= ((board.pieces(Piece::Rook) & *theirs).popcnt() * 525) as i16;

    cp += ((board.pieces(Piece::Queen) & *mine).popcnt() * 1000) as i16;
    cp -= ((board.pieces(Piece::Queen) & *theirs).popcnt() * 1000) as i16;

    cp
}
