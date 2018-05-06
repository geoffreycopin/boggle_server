use super::*;
use std::{
    io::prelude::*,
    sync::atomic::AtomicUsize,
};

pub enum Request {
    Login(String),
    Logout(String),
    Found(String, String),
    Chat(String, String),
    ChatAll(String),
}

pub struct Server {
    game: Game<CloneableWriter>,
    logger: Sender<LogMsg>,
    nb_players: Mutex<usize>,
}

impl Server {
    pub fn new<U: Dict  +'static>(dict: U, logger: Sender<LogMsg>) -> Server {
        let players = Players::new();
        let board = Board::new(true);
        let game = Game::new(players, board, dict);
        Server { game, logger, nb_players: Mutex::new(0) }
    }

    pub fn nb_players(&self) -> usize {
        self.nb_players.lock().unwrap().clone()
    }

    pub fn set_nb_users(&self, nb: usize) {
        *self.nb_players.lock().unwrap() = nb;
    }

    pub fn start_game_session(&self) {
        self.game.start_session();
        println!("Start session");
    }

    pub fn end_game_session(&self) {
        self.game.end_session();
        println!("End Session");
    }

    pub fn new_game_turn(&self) {
        self.game.new_turn();
        println!("start turn");
    }

    pub fn end_game_turn(&self) {
        self.game.end_turn();
    }

    pub fn handle_client_request(&self, request: &str, username: &str, mut stream: CloneableWriter) {
        let result = parse_request(request).and_then(|r| {
            match r {
                Request::Login(name) => self.login(&name, stream),
                Request::Logout(name) => self.logout(&name, stream),
                Request::Found(word, trajectory) =>
                    self.found(&username, &mut stream, &word, &trajectory),
                Request::Chat(to, message) => self.chat(username, &to, &message),
                Request::ChatAll(message) => self.chat_all(username, &message),
            }
        });
        if let Err(e) = result {
            self.log(LogMsg::err(e))
        }
    }

    pub fn login(&self, username: &str, writer: CloneableWriter) -> Result<(), ServerError> {
        self.game.login(username, writer.clone())
            .map(|_|  { self.log(LogMsg::login(username)); *self.nb_players.lock().unwrap() += 1 })
            .map_err(|e| { writer.shutdown(); e })
    }

    pub fn logout(&self, username: &str, writer: CloneableWriter) -> Result<(), ServerError> {
        self.game.logout(username).map(|_| {
            *self.nb_players.lock().unwrap() -= 1;
            writer.shutdown();
            self.log(LogMsg::logout(username))
        })
    }

    pub fn found(&self, username: &str, writer: &mut CloneableWriter, word: &str, trajectory: &str)
             -> Result<(), ServerError>
    {
        self.game.found(username, word, trajectory)
            .map(|_| {
                writer.write(format!("MVALIDE/{}/\n", word).as_bytes())
                    .expect("Cannot write response");
                self.log(LogMsg::Accepted(username.to_string(), word.to_string()))
            })
            .map_err(|e| {
                writer.write(format!("MINVALIDE/{}/\n", e).as_bytes())
                    .expect("Cannot write response");
                e
            })
    }

    pub fn chat(&self, sender: &str, receiver: &str, msg: &str) -> Result<(), ServerError>
    {
        self.game.chat(sender, receiver, msg).map(|_| {
            self.log(LogMsg::message_sent(sender, receiver, msg));
        })
    }

    pub fn chat_all(&self, sender: &str, message: &str) -> Result<(), ServerError> {
        self.game.chat_all(message).map(|_| {
            self.log(LogMsg::global_message(sender, message));
        })
    }

    pub fn remove_user_if_connected(&self, username: &str) {
        if self.game.is_connected(username) {
            if let Err(e) = self.game.logout(username) {
                eprintln!("Error while logging out: {}", e)
            }
        }
    }

    pub fn log(&self, msg: LogMsg) {
        self.logger.send(msg).unwrap()
    }
}

unsafe impl Sync for Server { }
