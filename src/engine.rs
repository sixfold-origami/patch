use anyhow::Context;
use rayon::iter::ParallelIterator;
use std::{cmp::Ordering, str::FromStr, time::Instant};

use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece};
use rayon::iter::{IntoParallelIterator, ParallelBridge};
use uci_parser::{UciInfo, UciResponse, UciScore, UciSearchOptions};

use crate::score::Score;

#[derive(Debug, Default)]
pub struct Engine {
    debug: bool,

    board: Board,

    stop_time: Option<Instant>,
    current_search_depth: u8,
    best_move_found: Option<ChessMove>,
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
    ///
    /// Resets everything except the [`Engine::debug()`] flag
    pub fn reset_game(&mut self) {
        *self = Self {
            debug: self.debug,
            ..Default::default()
        };
    }

    /// Resets internal search parameters and flags for a new search
    ///
    /// E.g. the best move found, the current search depth, etc.
    fn reset_search_params(&mut self) {
        self.stop_time = None;
        self.current_search_depth = 1;
        self.best_move_found = None;
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
        // Setup board
        let mut board = if let Some(fen) = fen {
            Board::from_str(fen).map_err(|e| anyhow::Error::msg(e))?
        } else {
            Board::default()
        };

        moves.for_each(|mv| board = board.make_move_new(mv));
        self.board = board;

        // Clean up for the upcoming search
        // We do this here, because we're allowed to block while setting up,
        // and this way we don't use up precious search time
        self.reset_search_params();

        Ok(())
    }

    /// Searches for the best move on the position setup in [`Engine::set_position`]
    ///
    /// If [`Engine::set_position`] is not called, then the default chess starting position is used
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
                let (time, inc) = match self.board.side_to_move() {
                    Color::White => (options.wtime, options.winc),
                    Color::Black => (options.btime, options.binc),
                };

                if let Some(time) = time {
                    // Basic thinking time hueristic
                    let thinking_time = if let Some(inc) = inc {
                        time / 20 + inc / 2
                    } else if let Some(movestogo) = options.movestogo {
                        time / movestogo
                    } else {
                        unimplemented!("Got unimplemented time control options");
                    };

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
        loop {
            let eval = self.evaluate_board(&self.board, 0);

            if !eval.terminated_early {
                let eval_mv = eval
                    .mv
                    .context("Asked to search on a position with no legal moves")?;

                println!(
                    "{}",
                    UciResponse::info(
                        UciInfo::new()
                            .score(UciScore::from(eval.score))
                            .pv([eval_mv.to_string()])
                            .depth(self.current_search_depth)
                    )
                );

                // TODO: we can still do this on early termination if the tree search is ordered based on previous search depths
                // TODO: handle stop command if stop_time is None
                self.best_move_found = Some(eval_mv);

                // Deeper we go
                self.current_search_depth += 1;
            } else {
                // We are done here
                return self
                    .best_move_found
                    .context("Failed to search even a single depth level");
            }
        }
    }

    /// Evaluates the provided board, assuming we are up to move
    ///
    /// Branches based on moves if possible.
    /// Returns the score for this position, and the best move it found,
    /// as long as we are not in a terminal case (recursion limit, stalemate, or checkmate).
    fn evaluate_board(&self, board: &Board, depth: u8) -> BoardEvaluation {
        match board.status() {
            BoardStatus::Checkmate => {
                // We lost :(
                BoardEvaluation::score(Score::Mate(0))
            }
            BoardStatus::Stalemate => BoardEvaluation::score(Score::cp(0)),
            BoardStatus::Ongoing => {
                if depth == self.current_search_depth {
                    // Terminate at max depth
                    // Hueristic based on material
                    BoardEvaluation::score(self.material_hueristic(board))
                } else if self
                    .stop_time
                    .map(|st| Instant::now() > st)
                    .unwrap_or_default()
                {
                    // Early termination on time
                    // Hueristic based on material
                    BoardEvaluation::score_early(self.material_hueristic(board))
                } else {
                    // Down the tree we go
                    let mut iter = MoveGen::new_legal(board);

                    // This will always return some non-identity value,
                    // as long as the above iterator has at least one valid move.
                    // This is always the case, because the cases where no moves are available (mates)
                    // are handled above
                    (&mut iter)
                        .par_bridge()
                        .into_par_iter()
                        .map(|mv| {
                            let next = board.make_move_new(mv);
                            BoardEvaluation::from_child(self.evaluate_board(&next, depth + 1), mv)
                        })
                        .reduce(
                            || BoardEvaluation::min(),
                            |acc, e| {
                                if e > acc {
                                    BoardEvaluation::new(
                                        e.mv,
                                        e.score,
                                        // Early termination propagates upward
                                        e.terminated_early || acc.terminated_early,
                                    )
                                } else {
                                    acc
                                }
                            },
                        )
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

/// Return value of [`Engine::evaluate_board`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoardEvaluation {
    /// The best move found in this subtree
    mv: Option<ChessMove>,
    /// The score of this subtree
    score: Score,
    /// Whether this subtree was terminated early,
    /// such as from a stop command or from running out of time
    terminated_early: bool,
}

impl BoardEvaluation {
    /// Constructs a new [`BoardEvaluation`]
    fn new(mv: Option<ChessMove>, score: Score, terminated_early: bool) -> Self {
        Self {
            mv,
            score,
            terminated_early,
        }
    }

    /// Constructs a [`BoardEvaluation`] from an evaluation coming out of a subtree
    ///
    /// This means that we must:
    /// - Flip the score, as children evaluate from their perspective
    /// - Paste in the move that got us from our board to the child board
    fn from_child(child: Self, mv: ChessMove) -> Self {
        Self {
            mv: Some(mv),
            score: child.score.flip(),
            // If they terminated early, then so did we, technically
            terminated_early: child.terminated_early,
        }
    }

    /// Constructs a new [`BoardEvaluation`] when only the score is known,
    /// such as in mating positions and stalemates.
    ///
    /// These positions are *terminal* inherently, so they are never considered an early termination
    fn score(score: Score) -> Self {
        Self {
            mv: None,
            score,
            terminated_early: false,
        }
    }

    /// Constructs a new [`BoardEvaluation`] for an early termination, using the score hueristic
    fn score_early(score: Score) -> Self {
        Self {
            mv: None,
            score,
            terminated_early: true,
        }
    }

    /// Constructs a [`Self`] which is always worse (lower than) than every other [`Self`]
    ///
    /// This is used as an identiy value when computing the best of a set of evaluations
    fn min() -> Self {
        Self {
            mv: None,
            score: Score::Mate(0),
            terminated_early: false,
        }
    }
}

impl Ord for BoardEvaluation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for BoardEvaluation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
