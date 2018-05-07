use super::errors::ServerError;

use std::{
    sync::mpsc::Receiver,
    fmt,
};

#[derive(Debug)]
pub enum LogMsg {
    Login(String),
    Logout(String),
    Error(ServerError),
    Accepted(String, String),
    MessageSent(String, String, String),
    GlobalMessage(String, String),
    SessionStart,
    SessionEnd,
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

    pub fn message_sent(sender: &str, receiver: &str, message: &str) -> LogMsg {
        LogMsg::MessageSent(sender.to_string(), receiver.to_string(), message.to_string())
    }

    pub fn global_message(sender: &str, message: &str)  -> LogMsg {
        LogMsg::GlobalMessage(sender.to_string(), message.to_string())
    }
}

impl fmt::Display for LogMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &LogMsg::Login(ref name) => write!(f, "{} vient de se connecter.", name),
            &LogMsg::Logout(ref name) => write!(f, "{} vient de se déconnecter.", name),
            &LogMsg::Accepted(ref name, ref word) => write!(f, "Le mot {} soumis par {} a été accepté.", word, name),
            &LogMsg::Error(ref e) => write!(f, "Erreur: {}", e),
            &LogMsg::MessageSent(ref s, ref r, ref m) =>
                write!(f, "Le message <{}> soumis par {} a été envoyé à {}.", m, s , r),
            &LogMsg::GlobalMessage(ref user, ref message) =>
                write!(f, "Le message <{}> soumis par {} a été envoyé à tous les utilisateurs.",
                       message, user),
            &LogMsg::SessionStart => write!(f, "Début de la session."),
            &LogMsg::SessionEnd => write!(f, "Fin de la session."),
        }
    }
}

pub fn log(commands: Receiver<LogMsg>) {
    for cmd in commands {
        println!("{}", cmd)
    }
}