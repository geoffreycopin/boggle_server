use super::errors::ServerError;

use std::{
    sync::mpsc::Receiver,
    fmt,
};

#[derive(Debug)]
pub enum LogCommands {
    Login(String),
    Error(ServerError)
}

impl fmt::Display for LogCommands {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &LogCommands::Login(ref name) => write!(f, "{} vient de se connecter", name),
            &LogCommands::Error(ref e) => write!(f, "Erreur: {}", e)
        }
    }
}

pub fn log(commands: Receiver<LogCommands>) {
    for cmd in commands {
        println!("{}", cmd)
    }
}