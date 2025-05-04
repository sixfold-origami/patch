use std::{error::Error, io::stdin};

use vampirc_uci::{CommunicationDirection, UciMessage, parse_with_unknown};

mod engine;

fn main() -> Result<(), Box<dyn Error>> {
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
                UciMessage::Debug(_) => unimplemented!(),
                UciMessage::IsReady => {
                    // Everything is blocking, so by the time we read this messages, we're ready
                    // TODO: make it so that it's not all blocking
                    println!("{}", UciMessage::ReadyOk);
                }
                UciMessage::Position {
                    startpos,
                    fen,
                    moves: _moves,
                } => {}
                UciMessage::SetOption { .. } => unimplemented!(),
                UciMessage::UciNewGame => unimplemented!(),
                UciMessage::Stop => unimplemented!(),
                UciMessage::PonderHit => unimplemented!(),
                UciMessage::Go { .. } => unimplemented!(),
                UciMessage::Quit => return Ok(()),
                UciMessage::Register { .. } => {
                    // We don't perform registration, so this is a NOP
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
