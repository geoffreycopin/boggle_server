use super::errors::ServerError;

use std::{
    sync::mpsc::Receiver,
    fmt,
};

#[derive(Debug)]
pub enum LogMsg {
    Login(String),
    Logout(String),
    Error(ServerError)
}

impl LogMsg {
    pub fn login(name: &str) -> LogMsg {
        LogMsg::Login(name.to_string())
    }

    pub fn logout(name: &str) -> LogMsg {
        LogMsg::Logout(name.to_string())
    }

    pub fn err(e: ServerError) -> LogMsg {
        LogMsg::Error(e)
    }
}

impl fmt::Display for LogMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &LogMsg::Login(ref name) => write!(f, "{} vient de se connecter.", name),
            &LogMsg::Logout(ref name) => write!(f, "{} vient de se déconnecter.", name),
            &LogMsg::Error(ref e) => write!(f, "Erreur: {}", e)
        }
    }
}

pub fn log(commands: Receiver<LogMsg>) {
    for cmd in commands {
        println!("{}", cmd)
    }
}