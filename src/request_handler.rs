use super::{
    log::LogCommands,
    game::Game,
    players::Players
};

use std::{
    io::Write,
    sync::mpsc::Sender,
};


pub struct RequestHandler<T: Write> {
    game: Game,
    players: Players<T>,
    log: Sender<LogCommands>,
}