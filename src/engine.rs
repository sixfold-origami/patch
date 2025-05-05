use std::str::FromStr;

use chess::{Board, BoardStatus, ChessMove, EMPTY, Game, MoveGen, Piece};
use rand::Rng;

const DEPTH_LIMIT: u8 = 4;

#[derive(Debug)]
pub struct Engine {
    game: Game,
    debug: bool,
}

impl Engine {
    /// Sets the debug flag
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Whether debugging is enabled
    pub fn debug(&self) -> bool {
        self.debug
    }

    /// Resets the internal state for a new game
    pub fn reset_game(&mut self) {
        self.game = Game::new();
    }

    /// Sets the board to the given position
    ///
    /// If `fen` is `None`, then the default starting position is used
    /// TODO: Try to reuse the existing game if possible
    pub fn set_position(
        &mut self,
        fen: Option<&str>,
        moves: impl Iterator<Item = ChessMove>,
    ) -> Result<(), anyhow::Error> {
        self.game = if let Some(fen) = fen {
            Game::from_str(fen).map_err(|e| anyhow::Error::msg(e))?
        } else {
            Game::new()
        };

        moves.for_each(|mv| {
            self.game.make_move(mv);
        });

        Ok(())
    }

    pub fn best_move(&self) -> ChessMove {
        let board = self.game.current_position();
        let mut iter = MoveGen::new_legal(&board);

        let targets = board.color_combined(!board.side_to_move());
        iter.set_iterator_mask(*targets);

        if let Some(mv) = iter.next() {
            return mv;
        }

        iter.set_iterator_mask(!EMPTY);

        let moves: Vec<_> = iter.collect();
        if moves.is_empty() {
            panic!("No legal moves");
        }
        let mut rng = rand::rng();
        moves[rng.random_range(0..moves.len())]
    }

    pub fn search(&self) -> ChessMove {
        let board = self.game.current_position();
        let mut iter = MoveGen::new_legal(&board);

        let targets = board.color_combined(!board.side_to_move());
        iter.set_iterator_mask(*targets);

        if let Some(best_capture) = (&mut iter)
            .map(|mv| (mv, self.evaluate_board(&board, mv, 0)))
            .reduce(|(original_mv, max), (new_mv, score)| {
                if score > max {
                    (new_mv, score)
                } else {
                    (original_mv, max)
                }
            })
        {
            return best_capture.0;
        };

        iter.set_iterator_mask(!EMPTY);

        if let Some(best_move) = (&mut iter)
            .map(|mv| (mv, self.evaluate_board(&board, mv, 0)))
            .reduce(|(original_mv, max), (new_mv, score)| {
                if score > max {
                    (new_mv, score)
                } else {
                    (original_mv, max)
                }
            })
        {
            return best_move.0;
        };

        unreachable!("No legal moves!");
    }

    /// Evaluates making `move` on `board` as if we are making this move
    fn evaluate_board(&self, board: &Board, mv: ChessMove, depth: u8) -> f32 {
        let next = board.make_move_new(mv);

        match next.status() {
            BoardStatus::Checkmate => {
                // We win
                return 100.;
            }
            BoardStatus::Stalemate => return 0.,
            BoardStatus::Ongoing => {
                if depth == DEPTH_LIMIT {
                    // Hueristic based on material

                    // We just moved, so their pieces are the side to move
                    let theirs = board.color_combined(board.side_to_move());
                    let mine = board.color_combined(!board.side_to_move());

                    // Get pieces and do sums
                    let mut my_score = 0;
                    let mut their_score = 0;

                    my_score += (board.pieces(Piece::Pawn) & *mine).0.count_ones();
                    their_score += (board.pieces(Piece::Pawn) & *theirs).0.count_ones();

                    my_score += (board.pieces(Piece::Knight) & *mine).0.count_ones() * 3;
                    their_score += (board.pieces(Piece::Knight) & *theirs).0.count_ones() * 3;

                    my_score += (board.pieces(Piece::Bishop) & *mine).0.count_ones() * 3;
                    their_score += (board.pieces(Piece::Bishop) & *theirs).0.count_ones() * 3;

                    my_score += (board.pieces(Piece::Rook) & *mine).0.count_ones() * 5;
                    their_score += (board.pieces(Piece::Rook) & *theirs).0.count_ones() * 5;

                    my_score += (board.pieces(Piece::Queen) & *mine).0.count_ones() * 9;
                    their_score += (board.pieces(Piece::Queen) & *theirs).0.count_ones() * 9;

                    return (my_score - their_score) as f32;
                } else {
                    // Down the tree we go
                    let mut iter = MoveGen::new_legal(&board);

                    // Search non-capture moves first, because we're minimizing anyway
                    let targets = board.color_combined(!board.side_to_move());
                    iter.set_iterator_mask(!*targets);

                    if let Some(min) = (&mut iter)
                        .map(|mv| self.evaluate_board(&board, mv, depth + 1))
                        .reduce(|min, score| min.min(score))
                    {
                        return min;
                    };

                    // Search everyting else (capture moves)
                    iter.set_iterator_mask(!EMPTY);

                    if let Some(min) = (&mut iter)
                        .map(|mv| self.evaluate_board(&board, mv, depth + 1))
                        .reduce(|min, score| min.min(score))
                    {
                        return min;
                    };

                    unreachable!("No legal moves! Handled above.");
                }
            }
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            game: Game::new(),
            debug: Default::default(),
        }
    }
}
