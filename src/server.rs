use super::{
    log::*,
    game::*,
    players::*,
    errors::ServerError,
};

use std::{
    io::{BufReader, BufRead},
    sync::{RwLock, mpsc::{Sender, Receiver}, Arc},
    thread,
    marker::Sync,
    net::TcpStream,
};

pub type LogChan = Sender<LogCommands>;
pub type ClientStream = (BufReader<TcpStream>, TcpStream);

pub enum Request {
    Login(String),
}

pub struct Server {
    game: RwLock<Game>,
    players: RwLock<Players<TcpStream>>,
    log: LogChan,
}

impl Server {
    pub fn new(log: LogChan, game: Game, players: Players<TcpStream>) -> Self {
        Server {
            game: RwLock::new(game),
            players: RwLock::new(players),
            log
        }
    }

    fn handle_client_request(&self, request: &str, stream: TcpStream) {
        let result = parse_request(request).and_then(|r| {
            match r {
                Request::Login(username) => self.login(&username, stream)
            }
        });
        result.map_err(|e| {
            self.log.send(LogCommands::Error(e.clone())).unwrap();
        });
    }

    fn login(&self, username: &str, writer: TcpStream) -> Result<(), ServerError> {
        let mut guard = self.players.write().unwrap();
        guard.login(username, writer).map(|_|
            self.log.send(LogCommands::Login(username.to_string())).unwrap()
        )
    }
}

unsafe impl Sync for Server { }

pub fn run(server: Server, streams: Receiver<TcpStream>) {
    let server = Arc::new(server);

    for sock in streams {
        let s = server.clone();
        thread::spawn(move || {
            let reader = BufReader::new(sock.try_clone().unwrap());
            for request in reader.lines() {
                if let Ok(r) = request {
                    s.handle_client_request(&r, sock.try_clone().unwrap());
                }
            }
        });
    }
}

fn parse_request(req: &str) -> Result<Request, ServerError> {
    let components: Vec<&str> = req.split("/").collect();
    let err = ServerError::bad_request(req.to_string());

    let request = match components.get(0).ok_or(err.clone())? {
        &"CONNEXION" => parse_connexion(&components),
        _ => Err(())
    };

    request.map_err(|_| err)
}

fn parse_connexion(components: &[&str]) -> Result<Request, ()> {
    let username = components.get(1).ok_or(())?;
    Ok(Request::Login(username.to_string()))
}