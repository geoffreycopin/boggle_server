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

impl<T: Write> RequestHandler<T> {
    pub fn new(log: Sender<LogCommands>) -> Self {
        RequestHandler {
            game: Game::new(log.clone()),
            players: Players::new(),
            log,
        }
    }
}