use anyhow::Context;
use rayon::iter::ParallelIterator;
use std::{str::FromStr, time::Instant};

use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square};
use rayon::iter::{IntoParallelIterator, ParallelBridge};
use uci_parser::{UciInfo, UciResponse, UciScore, UciSearchOptions};

use crate::score::Score;

#[derive(Debug, Default)]
pub struct Engine {
    debug: bool,

    board: Board,

    stop_time: Option<Instant>,
    current_search_depth: u8,
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
        self.board = Board::default();
    }

    /// Sets the board to the given position
    ///
    /// If `fen` is `None`, then the default starting position is used
    /// TODO: Try to reuse the existing board if possible?
    pub fn set_position(
        &mut self,
        fen: Option<&str>,
        moves: impl Iterator<Item = ChessMove>,
    ) -> Result<(), anyhow::Error> {
        let mut board = if let Some(fen) = fen {
            Board::from_str(fen).map_err(|e| anyhow::Error::msg(e))?
        } else {
            Board::default()
        };

        moves.for_each(|mv| board = board.make_move_new(mv));
        self.board = board;

        Ok(())
    }

    pub fn search(&mut self, options: UciSearchOptions) -> anyhow::Result<ChessMove> {
        // Determine stop time
        if !options.infinite {
            // In infinite mode, we search until told to stop
            // Otherwise, we figure out our time control

            if let Some(movetime) = options.movetime {
                // Search for the provided duration
                self.stop_time = Some(
                    Instant::now()
                        .checked_add(movetime)
                        .context("Failed to add provided movetime to current instant")?,
                );
            } else {
                if let (Some(time), Some(inc)) = match self.board.side_to_move() {
                    Color::White => (options.wtime, options.winc),
                    Color::Black => (options.btime, options.binc),
                } {
                    // Basic thinking time hueristic
                    let thinking_time = time / 20 + inc / 2;

                    self.stop_time = Some(
                        Instant::now()
                            .checked_add(thinking_time)
                            .context("Failed to add thinking time to current instant")?,
                    );
                } else {
                    unimplemented!("Got unimplemented time control options");
                }
            }
        }

        // Search
        self.current_search_depth = 1;

        loop {
            println!(
                "{}",
                UciResponse::info(UciInfo::new().depth(self.current_search_depth))
            );

            let (mv, score) = self.evaluate_board(&self.board, 1);

            println!(
                "{}",
                UciResponse::info(UciInfo::new().score(UciScore::from(score)))
            );

            if self.stop_time.is_some() && self.stop_time.unwrap() < Instant::now() {
                // Out of time, spit out what we got
                // TODO: handle stop command if stop_time is None
                return mv.context("Asked to search on a position with no legal moves");
            } else {
                self.current_search_depth += 1;
            }
        }
    }

    /// Evaluates the provided board, assuming we are up to move
    ///
    /// Branches based on moves if possible.
    /// Returns the score for this position, and the best move it found,
    /// as long as we are not in a terminal case (recursion limit, stalemate, or checkmate).
    fn evaluate_board(&self, board: &Board, depth: u8) -> (Option<ChessMove>, Score) {
        match board.status() {
            BoardStatus::Checkmate => {
                // We lost :(
                (None, Score::mate(0))
            }
            BoardStatus::Stalemate => {
                return (None, Score::cp(0));
            }
            BoardStatus::Ongoing => {
                if depth == self.current_search_depth {
                    // Hueristic based on material
                    (None, self.material_hueristic(board))
                } else {
                    // Down the tree we go
                    let mut iter = MoveGen::new_legal(board);

                    let (mv, score) = (&mut iter)
                        .par_bridge()
                        .into_par_iter()
                        .map(|mv| {
                            let next = board.make_move_new(mv);
                            (mv, self.evaluate_board(&next, depth + 1).1.flip()) // Scored as opponent
                        })
                        .reduce(
                            // Only the score is used for accumulation, so the move can be anything
                            // M0 is the "lowest" score, so it will never be selected
                            || (ChessMove::new(Square::A1, Square::A1, None), Score::mate(0)),
                            |(acc_mv, acc_sc), (mv, sc)| {
                                if sc > acc_sc {
                                    (mv, sc)
                                } else {
                                    (acc_mv, acc_sc)
                                }
                            },
                        );

                    // This will always be some non-identity value,
                    // as long as the above iterator has at least one valid move.
                    // This is always the case, because the cases where no moves are available (mates)
                    // are handled above
                    return (Some(mv), score);
                }
            }
        }
    }

    /// Scores the provided board based on material counts, assuming that we are up to move
    fn material_hueristic(&self, board: &Board) -> Score {
        let mine = board.color_combined(board.side_to_move());
        let theirs = board.color_combined(!board.side_to_move());

        // Get pieces and do sums
        let mut pawns: i16 = 0;

        pawns += ((board.pieces(Piece::Pawn) & *mine).0.count_ones()) as i16;
        pawns -= ((board.pieces(Piece::Pawn) & *theirs).0.count_ones()) as i16;

        pawns += ((board.pieces(Piece::Knight) & *mine).0.count_ones() * 3) as i16;
        pawns -= ((board.pieces(Piece::Knight) & *theirs).0.count_ones() * 3) as i16;

        pawns += ((board.pieces(Piece::Bishop) & *mine).0.count_ones() * 3) as i16;
        pawns -= ((board.pieces(Piece::Bishop) & *theirs).0.count_ones() * 3) as i16;

        pawns += ((board.pieces(Piece::Rook) & *mine).0.count_ones() * 5) as i16;
        pawns -= ((board.pieces(Piece::Rook) & *theirs).0.count_ones() * 5) as i16;

        pawns += ((board.pieces(Piece::Queen) & *mine).0.count_ones() * 9) as i16;
        pawns -= ((board.pieces(Piece::Queen) & *theirs).0.count_ones() * 9) as i16;

        Score::cp(pawns * 100)
    }
}
