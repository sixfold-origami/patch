use std::str::FromStr;

use chess::Game;

#[derive(Debug)]
pub struct Engine {
    game: Game,
    debug: bool,
}

impl Engine {
    /// Resets the board position to the starting position
    pub fn starting_position(&mut self) {
        self.game = Game::new();
    }

    /// Resets the board to the given position
    pub fn position(&mut self, fen: &str) -> Result<(), anyhow::Error> {
        self.game = Game::from_str(fen).map_err(|e| failure::Error::from(e).compat())?;

        Ok(())
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
