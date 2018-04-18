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

pub fn log(commands: Receiver<LogCommands>) {
    for cmd in commands {
        println!("{:#?}", cmd)
    }
}