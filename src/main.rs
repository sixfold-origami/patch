use std::{error::Error, io::stdin};

use engine::Engine;
use vampirc_uci::{CommunicationDirection, UciMessage, parse_with_unknown};

pub mod engine;

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::default();

    for line in stdin().lines() {
        for m in parse_with_unknown(&line.unwrap())
            .into_iter()
            .filter(|m| m.direction() == CommunicationDirection::GuiToEngine)
        {
            match m {
                UciMessage::Uci => {
                    // Identify ourselves
                    println!("{}", UciMessage::id_name("Patch"));
                    println!("{}", UciMessage::id_author("sixfold"));
                    // Shake the nice GUI's hand
                    println!("{}", UciMessage::UciOk);
                }
                UciMessage::Debug(debug) => engine.set_debug(debug),
                UciMessage::IsReady => {
                    // Everything is blocking, so by the time we read this message, we're ready
                    // TODO: make it so that it's not all blocking
                    println!("{}", UciMessage::ReadyOk);
                }
                UciMessage::Position {
                    startpos,
                    fen,
                    moves,
                } => {
                    if startpos {
                        engine.set_starting_position(moves);
                    } else {
                        engine.set_position(fen.unwrap().as_str(), moves)?;
                    }
                }
                UciMessage::SetOption { .. } => unimplemented!(),
                UciMessage::UciNewGame | UciMessage::Stop => {
                    // TODO: NOP for now
                }
                UciMessage::PonderHit => unimplemented!(),
                UciMessage::Go {
                    time_control: _time_control,
                    search_control,
                } => {
                    if search_control.is_some() {
                        unimplemented!()
                    }

                    // We're too stupid to do a real search, but the benefit is that we can respond right away :clueless:
                    let mv = engine.best_move();
                    println!("{}", UciMessage::best_move(mv));
                }
                UciMessage::Quit => return Ok(()),
                UciMessage::Register { .. } => {
                    // We don't perform registration, so this is a NOP
                }
                UciMessage::Unknown(str, _) => {
                    println!("Unknown UCI message: {}", UciMessage::info_string(str))
                }
                _ => {
                    // EngineToGui messages
                    unreachable!()
                }
            }
        }
    }

    unreachable!()
}
