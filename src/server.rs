use super::{
    log::*,
    game::*,
    players::*,
    errors::ServerError,
};

use std::{
    io::{BufReader, BufWriter, Read, Write, BufRead},
    sync::{RwLock, mpsc::{Sender, Receiver}},
    thread,
    marker::Send,
    net::TcpStream,
};

pub type LogChan = Sender<LogCommands>;
pub type GameChan = Sender<GameCommand>;
pub type PlayersChan = Sender<PlayersCommand>;
pub type ClientStream = (BufReader<TcpStream>, TcpStream);

pub enum Request {
    PlayersRequest(PlayersCommand),
    GameRequest(GameCommand),
}

pub struct Server {
    game: GameChan,
    players: PlayersChan,
    log: LogChan,
}

impl Server {
    pub fn new(log: LogChan, game: GameChan, players: PlayersChan) -> Self {
        Server { game, players, log }
    }

    pub fn start(mut self, requests: Receiver<TcpStream>) {
        for r in requests {
            println!("Handling request !");
            let reader = BufReader::new(r.try_clone().unwrap());
            if let Err(e) = self.handle_request((reader, r)) {
                self.log.send(LogCommands::Error(e));
            }
        }
    }

    fn handle_request(&mut self, mut stream: ClientStream) -> Result<(), ServerError> {
        let mut req = String::new();
        stream.0.read_line(&mut req);

        match Server::parse_request(&req, stream)? {
            Request::GameRequest(cmd) => self.game.send(cmd).unwrap(),
            Request::PlayersRequest(cmd) => self.players.send(cmd).unwrap(),
        };

        Ok(())
    }

    fn parse_request(request: &str, stream: ClientStream) -> Result<Request, ServerError> {
        let components: Vec<&str> = request.split("/").collect();

        let req = match components.get(0) {
            Some(&"CONNEXION") => Server::parse_connexion_req(&components, stream),
            _ => Err(()),
        };

        req.or(Err(ServerError::bad_request(request.to_string())))
    }

    fn parse_connexion_req(components: &[&str], stream: ClientStream) -> Result<Request, ()> {
        let username = components.get(1).unwrap();
        Ok(Request::PlayersRequest(PlayersCommand::Login(username.to_string(), stream)))
    }
}