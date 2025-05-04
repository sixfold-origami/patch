use std::str::FromStr;

use chess::{ChessMove, EMPTY, Game, MoveGen};
use rand::Rng;

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

    /// Resets the board position to the starting position
    pub fn set_starting_position(&mut self, moves: Vec<ChessMove>) {
        self.game = Game::new();

        moves.into_iter().for_each(|mv| {
            self.game.make_move(mv);
        });
    }

    /// Resets the board to the given position
    pub fn set_position(&mut self, fen: &str, moves: Vec<ChessMove>) -> Result<(), anyhow::Error> {
        self.game = Game::from_str(fen).map_err(|e| failure::Error::from(e).compat())?;
        moves.into_iter().for_each(|mv| {
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
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            game: Game::new(),
            debug: Default::default(),
        }
    }
}
