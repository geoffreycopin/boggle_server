use super::errors::ServerError;

use std::{
    sync::mpsc::{channel, Sender, Receiver},
    thread,
};

#[derive(Debug)]
pub enum LogCommands {
    Players(String),
    Game(String),
    Error(ServerError)
}


pub fn start_logging() -> Sender<LogCommands> {
    let (sender, receiver) = channel();
    thread::spawn(move || log(receiver));
    sender
}

pub fn log(commands: Receiver<LogCommands>) {
    for cmd in commands {
        eprintln!("{:#?}", cmd)
    }
}