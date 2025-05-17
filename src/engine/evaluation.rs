use chess::{Board, Color, Piece, Square};
use tables::{
    ENDGAME_BISHOP_VALUE, ENDGAME_KING_VALUE, ENDGAME_KNIGHT_VALUE, ENDGAME_PAWN_VALUE,
    ENDGAME_QUEEN_VALUE, ENDGAME_ROOK_VALUE, MIDGAME_BISHOP_VALUE, MIDGAME_KING_VALUE,
    MIDGAME_KNIGHT_VALUE, MIDGAME_PAWN_VALUE, MIDGAME_QUEEN_VALUE, MIDGAME_ROOK_VALUE,
};

use crate::score::Score;

/// Evaluation heuristic based on material and piece positions
pub fn eval_heuristic(board: &Board) -> Score {
    let score = piece_table_eval(board);

    Score::cp(score)
}

/// Scores the provided board based on pure material value, assuming that we are up to move
#[allow(unused)]
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

/// Scores the provided board using piece [`tables`]
///
/// Pieces are given values based both on their material value and their position on the board
fn piece_table_eval(board: &Board) -> i16 {
    let phase = (board.pieces(Piece::Knight).popcnt()
        + board.pieces(Piece::Bishop).popcnt()
        + 2 * board.pieces(Piece::Rook).popcnt()
        + 4 * board.pieces(Piece::Queen).popcnt())
    .min(24) as i16; // Account for early promotion

    let inverse_phase = 24 - phase;

    let (mg_score, eg_score) = (0..64)
        .into_iter()
        .map(|i| {
            let square = Square::new(i);

            let piece = board.piece_on(square);
            let color = board.color_on(square);
            match (piece, color) {
                (None, None) => (0, 0), // No piece here, just return the identity
                (Some(piece), Some(color)) => {
                    let index = match board.side_to_move() {
                        Color::White => i,
                        // Piece tables are always relative to the current player,
                        // But the square indices are absolute (starting at A1).
                        // So, black must flip the index to get the right orientation.
                        Color::Black => i ^ 56,
                    } as usize;

                    let mg_score = match piece {
                        Piece::Pawn => MIDGAME_PAWN_VALUE[index],
                        Piece::Knight => MIDGAME_KNIGHT_VALUE[index],
                        Piece::Bishop => MIDGAME_BISHOP_VALUE[index],
                        Piece::Rook => MIDGAME_ROOK_VALUE[index],
                        Piece::Queen => MIDGAME_QUEEN_VALUE[index],
                        Piece::King => MIDGAME_KING_VALUE[index],
                    };

                    let eg_score = match piece {
                        Piece::Pawn => ENDGAME_PAWN_VALUE[index],
                        Piece::Knight => ENDGAME_KNIGHT_VALUE[index],
                        Piece::Bishop => ENDGAME_BISHOP_VALUE[index],
                        Piece::Rook => ENDGAME_ROOK_VALUE[index],
                        Piece::Queen => ENDGAME_QUEEN_VALUE[index],
                        Piece::King => ENDGAME_KING_VALUE[index],
                    };

                    if color == board.side_to_move() {
                        (mg_score, eg_score)
                    } else {
                        (-mg_score, -eg_score)
                    }
                }
                _ => unreachable!(),
            }
        })
        .reduce(|(mg_acc, eg_acc), (mg_score, eg_score)| (mg_acc + mg_score, eg_acc + eg_score))
        .unwrap();

    (mg_score * phase + eg_score * inverse_phase) / 24
}

/// Contains all the values for the piece tables and material values
///
/// Tables are from [PeSTO](https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function)
mod tables {
    use super::*;

    /// Centipawn values for each piece during the midgame
    ///
    /// Actual values are interpolated between mid and endgame, based on how much material is on the board
    const fn midgame_material_values(piece: Piece) -> i16 {
        match piece {
            Piece::Pawn => 82,
            Piece::Knight => 337,
            Piece::Bishop => 365,
            Piece::Rook => 477,
            Piece::Queen => 1025,
            Piece::King => 0, // Kings are handled by the mating rules
        }
    }

    /// Centipawn values for each piece during the endgame
    ///
    /// Actual values are interpolated between mid and endgame, based on how much material is on the board
    const fn endgame_material_values(piece: Piece) -> i16 {
        match piece {
            Piece::Pawn => 94,
            Piece::Knight => 281,
            Piece::Bishop => 297,
            Piece::Rook => 512,
            Piece::Queen => 936,
            Piece::King => 0, // Kings are handled by the mating rules
        }
    }

    /// Positional value for a pawn in the midgame
    #[rustfmt::skip]
    const MIDGAME_PAWN_POSITION_VALUE: [i16; 64] = [
         0,   0,   0,   0,   0,   0,  0,   0,
         -35,  -1, -20, -23, -15,  24, 38, -22,
         -26,  -4,  -4, -10,   3,   3, 33, -12,
         -27,  -2,  -5,  12,  17,   6, 10, -25,
         -14,  13,   6,  21,  23,  12, 17, -23,
         -6,   7,  26,  31,  65,  56, 25, -20,
         98, 134,  61,  95,  68, 126, 34, -11,
         0,   0,   0,   0,   0,   0,  0,   0,
    ];

    /// Positional value for a pawn in the endgame
    #[rustfmt::skip]
    const ENDGAME_PAWN_POSITION_VALUE: [i16; 64] = [
          0,   0,   0,   0,   0,   0,   0,   0,
         13,   8,   8,  10,  13,   0,   2,  -7,
          4,   7,  -6,   1,   0,  -5,  -1,  -8,
         13,   9,  -3,  -7,  -7,  -8,   3,  -1,
         32,  24,  13,   5,  -2,   4,  17,  17,
         94, 100,  85,  67,  56,  53,  82,  84,
        178, 173, 158, 134, 147, 132, 165, 187,
          0,   0,   0,   0,   0,   0,   0,   0,
    ];

    /// Positional value for a knight in the midgame
    #[rustfmt::skip]
    const MIDGAME_KNIGHT_POSITION_VALUE: [i16; 64] = [
        -105, -21, -58, -33, -17, -28, -19,  -23,
         -29, -53, -12,  -3,  -1,  18, -14,  -19,
         -23,  -9,  12,  10,  19,  17,  25,  -16,
         -13,   4,  16,  13,  28,  19,  21,   -8,
          -9,  17,  19,  53,  37,  69,  18,   22,
         -47,  60,  37,  65,  84, 129,  73,   44,
         -73, -41,  72,  36,  23,  62,   7,  -17,
        -167, -89, -34, -49,  61, -97, -15, -107,
    ];

    /// Positional value for a knight in the endgame
    #[rustfmt::skip]
    const ENDGAME_KNIGHT_POSITION_VALUE: [i16; 64] = [
        -29, -51, -23, -15, -22, -18, -50, -64,
        -42, -20, -10,  -5,  -2, -20, -23, -44,
        -23,  -3,  -1,  15,  10,  -3, -20, -22,
        -18,  -6,  16,  25,  16,  17,   4, -18,
        -17,   3,  22,  22,  22,  11,   8, -18,
        -24, -20,  10,   9,  -1,  -9, -19, -41,
        -25,  -8, -25,  -2,  -9, -25, -24, -52,
        -58, -38, -13, -28, -31, -27, -63, -99,
    ];

    /// Positional value for a bishop in the midgame
    #[rustfmt::skip]
    const MIDGAME_BISHOP_POSITION_VALUE: [i16; 64] = [
        -33,  -3, -14, -21, -13, -12, -39, -21,
          4,  15,  16,   0,   7,  21,  33,   1,
          0,  15,  15,  15,  14,  27,  18,  10,
         -6,  13,  13,  26,  34,  12,  10,   4,
         -4,   5,  19,  50,  37,  37,   7,  -2,
        -16,  37,  43,  40,  35,  50,  37,  -2,
        -26,  16, -18, -13,  30,  59,  18, -47,
        -29,   4, -82, -37, -25, -42,   7,  -8,
    ];

    /// Positional value for a bishop in the endgame
    #[rustfmt::skip]
    const ENDGAME_BISHOP_POSITION_VALUE: [i16; 64] = [
        -23,  -9, -23,  -5, -9, -16,  -5, -17,
        -14, -18,  -7,  -1,  4,  -9, -15, -27,
        -12,  -3,   8,  10, 13,   3,  -7, -15,
         -6,   3,  13,  19,  7,  10,  -3,  -9,
         -3,   9,  12,   9, 14,  10,   3,   2,
          2,  -8,   0,  -1, -2,   6,   0,   4,
         -8,  -4,   7, -12, -3, -13,  -4, -14,
        -14, -21, -11,  -8, -7,  -9, -17, -24,
    ];

    /// Positional value for a bishop in the midgame
    #[rustfmt::skip]
    const MIDGAME_ROOK_POSITION_VALUE: [i16; 64] = [
        -19, -13,   1,  17, 16,  7, -37, -26,
        -44, -16, -20,  -9, -1, 11,  -6, -71,
        -45, -25, -16, -17,  3,  0,  -5, -33,
        -36, -26, -12,  -1,  9, -7,   6, -23,
        -24, -11,   7,  26, 24, 35,  -8, -20,
         -5,  19,  26,  36, 17, 45,  61,  16,
         27,  32,  58,  62, 80, 67,  26,  44,
         32,  42,  32,  51, 63,  9,  31,  43,
    ];

    /// Positional value for a bishop in the endgame
    #[rustfmt::skip]
    const ENDGAME_ROOK_POSITION_VALUE: [i16; 64] = [
        -9,  2,  3, -1, -5, -13,   4, -20,
        -6, -6,  0,  2, -9,  -9, -11,  -3,
        -4,  0, -5, -1, -7, -12,  -8, -16,
         3,  5,  8,  4, -5,  -6,  -8, -11,
         4,  3, 13,  1,  2,   1,  -1,   2,
         7,  7,  7,  5,  4,  -3,  -5,  -3,
        11, 13, 13, 11, -3,   3,   8,   3,
        13, 10, 18, 15, 12,  12,   8,   5,
    ];

    /// Positional value for a bishop in the midgame
    #[rustfmt::skip]
    const MIDGAME_QUEEN_POSITION_VALUE: [i16; 64] = [
        -1, -18,  -9,  10, -15, -25, -31, -50,
        -35,  -8,  11,   2,   8,  15,  -3,   1,
        -14,   2, -11,  -2,  -5,   2,  14,   5,
         -9, -26,  -9, -10,  -2,  -4,   3,  -3,
        -27, -27, -16, -16,  -1,  17,  -2,   1,
        -13, -17,   7,   8,  29,  56,  47,  57,
        -24, -39,  -5,   1, -16,  57,  28,  54,
        -28,   0,  29,  12,  59,  44,  43,  45,
    ];

    /// Positional value for a bishop in the endgame
    #[rustfmt::skip]
    const ENDGAME_QUEEN_POSITION_VALUE: [i16; 64] = [
        -33, -28, -22, -43,  -5, -32, -20, -41,
        -22, -23, -30, -16, -16, -23, -36, -32,
        -16, -27,  15,   6,   9,  17,  10,   5,
        -18,  28,  19,  47,  31,  34,  39,  23,
          3,  22,  24,  45,  57,  40,  57,  36,
        -20,   6,   9,  49,  47,  35,  19,   9,
        -17,  20,  32,  41,  58,  25,  30,   0,
         -9,  22,  22,  27,  27,  19,  10,  20,
    ];

    /// Positional value for a bishop in the midgame
    #[rustfmt::skip]
    const MIDGAME_KING_POSITION_VALUE: [i16; 64] = [
        -15,  36,  12, -54,   8, -28,  24,  14,
          1,   7,  -8, -64, -43, -16,   9,   8,
        -14, -14, -22, -46, -44, -30, -15, -27,
        -49,  -1, -27, -39, -46, -44, -33, -51,
        -17, -20, -12, -27, -30, -25, -14, -36,
         -9,  24,   2, -16, -20,   6,  22, -22,
         29,  -1, -20,  -7,  -8,  -4, -38, -29,
        -65,  23,  16, -15, -56, -34,   2,  13,
    ];

    /// Positional value for a bishop in the endgame
    #[rustfmt::skip]
    const ENDGAME_KING_POSITION_VALUE: [i16; 64] = [
        -53, -34, -21, -11, -28, -14, -24, -43,
        -27, -11,   4,  13,  14,   4,  -5, -17,
        -19,  -3,  11,  21,  23,  16,   7,  -9,
        -18,  -4,  21,  24,  27,  23,   9, -11,
         -8,  22,  24,  27,  26,  33,  26,   3,
         10,  17,  23,  15,  20,  45,  44,  13,
        -12,  17,  14,  17,  17,  38,  23,  11,
        -74, -35, -18, -18, -11,  15,   4, -17,
    ];

    /// Piece table for a pawn in the midgame, combining its inherent material value and its positional value
    pub const MIDGAME_PAWN_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = MIDGAME_PAWN_POSITION_VALUE[i] + midgame_material_values(Piece::Pawn);
            i += 1;
        }

        table
    };

    /// Piece table for a pawn in the endgame, combining its inherent material value and its positional value
    pub const ENDGAME_PAWN_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = ENDGAME_PAWN_POSITION_VALUE[i] + endgame_material_values(Piece::Pawn);
            i += 1;
        }

        table
    };

    /// Piece table for a knight in the midgame, combining its inherent material value and its positional value
    pub const MIDGAME_KNIGHT_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = MIDGAME_KNIGHT_POSITION_VALUE[i] + midgame_material_values(Piece::Knight);
            i += 1;
        }

        table
    };

    /// Piece table for a knight in the endgame, combining its inherent material value and its positional value
    pub const ENDGAME_KNIGHT_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = ENDGAME_KNIGHT_POSITION_VALUE[i] + endgame_material_values(Piece::Knight);
            i += 1;
        }

        table
    };

    /// Piece table for a bishop in the midgame, combining its inherent material value and its positional value
    pub const MIDGAME_BISHOP_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = MIDGAME_BISHOP_POSITION_VALUE[i] + midgame_material_values(Piece::Bishop);
            i += 1;
        }

        table
    };

    /// Piece table for a bishop in the endgame, combining its inherent material value and its positional value
    pub const ENDGAME_BISHOP_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = ENDGAME_BISHOP_POSITION_VALUE[i] + endgame_material_values(Piece::Bishop);
            i += 1;
        }

        table
    };

    /// Piece table for a rook in the midgame, combining its inherent material value and its positional value
    pub const MIDGAME_ROOK_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = MIDGAME_ROOK_POSITION_VALUE[i] + midgame_material_values(Piece::Rook);
            i += 1;
        }

        table
    };

    /// Piece table for a rook in the endgame, combining its inherent material value and its positional value
    pub const ENDGAME_ROOK_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = ENDGAME_ROOK_POSITION_VALUE[i] + endgame_material_values(Piece::Rook);
            i += 1;
        }

        table
    };

    /// Piece table for a queen in the midgame, combining its inherent material value and its positional value
    pub const MIDGAME_QUEEN_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = MIDGAME_QUEEN_POSITION_VALUE[i] + midgame_material_values(Piece::Queen);
            i += 1;
        }

        table
    };

    /// Piece table for a queen in the endgame, combining its inherent material value and its positional value
    pub const ENDGAME_QUEEN_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = ENDGAME_QUEEN_POSITION_VALUE[i] + endgame_material_values(Piece::Queen);
            i += 1;
        }

        table
    };

    /// Piece table for a king in the midgame, combining its inherent material value and its positional value
    pub const MIDGAME_KING_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = MIDGAME_KING_POSITION_VALUE[i] + midgame_material_values(Piece::King);
            i += 1;
        }

        table
    };

    /// Piece table for a king in the endgame, combining its inherent material value and its positional value
    pub const ENDGAME_KING_VALUE: [i16; 64] = {
        let (mut table, mut i) = ([0; 64], 0);

        while i < 64 {
            table[i] = ENDGAME_KING_POSITION_VALUE[i] + endgame_material_values(Piece::King);
            i += 1;
        }

        table
    };
}
