use super::{
    log::*,
    board::*,
    players::*,
    errors::ServerError,
};

use std::{
    io::{BufReader, BufRead, Write},
    sync::{RwLock, mpsc::{Sender, Receiver}, Arc},
    thread,
    marker::Sync,
    net::{TcpStream, Shutdown},
};

pub type LogChan = Sender<LogMsg>;

pub enum Request {
    Login(String),
    Logout(String),
}

pub struct Server {
    game: RwLock<Board>,
    players: RwLock<Players<TcpStream>>,
    logger: LogChan,
}

impl Server {
    pub fn new(logger: LogChan, game: Board, players: Players<TcpStream>) -> Self {
        Server {
            game: RwLock::new(game),
            players: RwLock::new(players),
            logger
        }
    }

    fn handle_client_request(&self, request: &str, stream: TcpStream) {
        let result = parse_request(request).and_then(|r| {
            match r {
                Request::Login(name) => self.login(&name, stream),
                Request::Logout(name) => self.logout(&name, stream),
            }
        });
        result.map_err(|e| {
            self.log(LogMsg::err(e))
        });
    }

    fn login(&self, username: &str, mut writer: TcpStream) -> Result<(), ServerError> {
        let mut guard = self.players.write().unwrap();
        let res =  guard.login(username, writer.try_clone().unwrap());
        if let Err(ref e) = res {
            writer.shutdown(Shutdown::Both).unwrap();
        } else {
            self.welcome(&mut writer, &guard.users());
            self.log(LogMsg::login(username))
        }
        res
    }

    fn welcome(&self, user_stream: &mut TcpStream, users: &[String]) {
        let game = self.game.read().unwrap();
        let welcome_str = game.welcome_str(&users);
        user_stream.write(welcome_str.as_bytes()).unwrap();
    }

    fn logout(&self, username: &str, writer: TcpStream) -> Result<(), ServerError> {
        writer.shutdown(Shutdown::Both);
        let mut guard = self.players.write().unwrap();
        guard.logout(username)
            .map(|_| self.log(LogMsg::logout(username)))
    }

    fn log(&self, msg: LogMsg) {
        self.logger.send(msg).unwrap()
    }

    fn scores_to_string(scores: Vec<(String, u32)>) -> String {
        scores.into_iter()
            .map(|(user, score)| format!("{}*{}", user, score))
            .fold(String::new(), |acc, val| acc + &val)
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
    let err = ServerError::bad_request(req);

    let request = match components.get(0).ok_or(err.clone())? {
        &"CONNEXION" => parse_connexion(&components),
        &"SORT" => parse_sort(&components),
        _ => Err(())
    };

    request.map_err(|_| err)
}

fn parse_connexion(components: &[&str]) -> Result<Request, ()> {
    let username = components.get(1).ok_or(())?;
    Ok(Request::Login(username.to_string()))
}

fn parse_sort(components: &[&str]) -> Result<Request, ()> {
    let username = components.get(1).ok_or(())?;
    Ok(Request::Logout(username.to_string()))
}