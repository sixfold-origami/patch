use std::{error::Error, io::stdin, str::FromStr};

use chess::ChessMove;
use engine::Engine;
use uci_parser::{UciCommand, UciResponse};

pub mod engine;
pub mod score;

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::default();

    for line in stdin().lines() {
        match line.unwrap().parse::<UciCommand>()? {
            UciCommand::Uci => {
                // Identify ourselves
                println!("{}", UciResponse::Name("Patch"));
                println!("{}", UciResponse::Author("sixfold"));
                // Shake the nice GUI's hand
                println!("{}", UciResponse::uciok());
            }
            UciCommand::Debug(debug) => engine.set_debug(debug),
            UciCommand::IsReady => {
                // Everything is blocking, so by the time we read this message, we're ready
                // TODO: make it so that it's not all blocking
                println!("{}", UciResponse::readyok());
            }
            UciCommand::SetOption { .. } => unimplemented!(),
            UciCommand::Register { .. } => {
                // We don't perform registration, so this is a NOP
            }
            UciCommand::UciNewGame => {
                engine.reset_game();
            }
            UciCommand::Position { fen, moves } => {
                let moves = moves
                    .into_iter()
                    .map(|s| ChessMove::from_str(&s).expect("Valid move"));

                engine.set_position(fen.as_ref().map(|s| s.as_str()), moves)?;
            }
            UciCommand::Go(options) => {
                // We're too stupid to do a real search, but the benefit is that we can respond right away :clueless:
                let mv = engine.search(options)?;
                println!(
                    "{}",
                    UciResponse::BestMove {
                        bestmove: Some(mv.to_string()),
                        ponder: None,
                    }
                );
            }
            UciCommand::Stop => {
                // NOP for now
            }
            UciCommand::PonderHit => unimplemented!(),
            UciCommand::Quit => return Ok(()),
        }
    }

    unreachable!()
}
