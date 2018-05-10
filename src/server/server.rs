use super::*;
use super::super::dict::LocalDict;
use std::io::prelude::*;

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
    nb_turn: u64,
    turn_duration: Duration,
    pause_duration: Duration,
}

impl Server {
    pub fn new(logger: Sender<LogMsg>) -> Server {
        let players = Players::new();
        let board = Board::new(true, vec![]);
        let game = Game::new(players, board, LocalDict::new());
        Server {
            game,
            logger,
            nb_players: Mutex::new(0),
            nb_turn: 10,
            turn_duration: Duration::from_secs(180),
            pause_duration: Duration::from_secs(15),
        }
    }

    pub fn with_game(mut self, game: Game<CloneableWriter>) -> Server {
        self.game = game;
        self
    }

    pub fn with_turn_duration(mut self, duration: Duration) -> Server {
        self.turn_duration = duration;
        self
    }

    pub fn with_pause_duration(mut self, duration: Duration) -> Server {
        self.pause_duration = duration;
        self
    }

    pub fn with_nb_turn(mut self, nb_turn: u64) -> Server {
        self.nb_turn = nb_turn;
        self
    }

    pub fn nb_turn(&self) -> u64 {
        self.nb_turn
    }

    pub fn turn_duration(&self) -> Duration {
        self.turn_duration
    }

    pub fn pause_duration(&self) -> Duration {
        self.pause_duration
    }

    pub fn nb_players(&self) -> usize {
        self.nb_players.lock().unwrap().clone()
    }

    /// Démarre une session de jeu.
    pub fn start_game_session(&self) {
        self.game.start_session();
        self.log(LogMsg::SessionStart);
    }

    /// Met fin à la session de jeu courante.
    pub fn end_game_session(&self) {
        self.game.end_session();
        self.log(LogMsg::SessionEnd);
    }

    /// Démarre un nouveau tour.
    pub fn new_game_turn(&self) {
        self.game.new_turn();
        println!("start turn");
    }

    /// Met fin au tour courant.
    pub fn end_game_turn(&self) {
        self.game.end_turn();
    }

    /// Taite la requête `request` de l'utlisateur `username`.
    /// La réponse éventuelle sera crite sur le stream `stream`.
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

    /// Enregsitre un nouvel utilisateur `username`.
    pub fn login(&self, username: &str, writer: CloneableWriter) -> Result<(), ServerError> {
        self.game.login(username, writer.clone())
            .map(|_|  { self.log(LogMsg::login(username)); *self.nb_players.lock().unwrap() += 1 })
            .map_err(|e| { writer.shutdown(); e })
    }

    /// Supprime l'utlisateur `username` et clos la connexion.
    pub fn logout(&self, username: &str, writer: CloneableWriter) -> Result<(), ServerError> {
        self.game.logout(username).map(|_| {
            *self.nb_players.lock().unwrap() -= 1;
            writer.shutdown();
            self.log(LogMsg::logout(username))
        })
    }

    /// Soumission du mot `word` de trajectoire `trajectory` par l'utilisateur `username`.
    pub fn found(&self, username: &str, writer: &mut CloneableWriter, word: &str, trajectory: &str)
             -> Result<(), ServerError>
    {
        self.game.found(username, word, trajectory)
            .map(|is_immediate| {
                if is_immediate {
                    writer.write(format!("MVALIDE/{}/\n", word).as_bytes())
                        .expect("Cannot write response");
                }
                self.log(LogMsg::accepted(username, word));
            })
            .map_err(|e| {
                if let ServerError::AlreadyPlayed {ref word, immediate: true} = e {
                    let msg = format!("PRI: le mot <{}> a déjà été joué !", word);
                    writer.write(format!("MINVALIDE/{}/\n", msg).as_bytes())
                        .expect("Cannot write response");
                }
                e
            })
    }

    /// Envoi du message `msg` à l'utilisateur `receiver` par l'utilisateur `sender`.
    pub fn chat(&self, sender: &str, receiver: &str, msg: &str) -> Result<(), ServerError>
    {
        self.game.chat(sender, receiver, msg).map(|_| {
            self.log(LogMsg::message_sent(sender, receiver, msg));
        })
    }

    /// Envoi du message `message à tous les utilisateurs`.
    pub fn chat_all(&self, sender: &str, message: &str) -> Result<(), ServerError> {
        self.game.chat_all(message).map(|_| {
            self.log(LogMsg::global_message(sender, message));
        })
    }

    /// Déconnecte l'utilisateur `username` s'il était connecté.
    pub fn remove_user_if_connected(&self, username: &str) {
        if self.game.is_connected(username) {
            if let Err(e) = self.game.logout(username) {
                eprintln!("Error while logging out: {}", e)
            }
        }
    }

    /// Envoie une commande au Logger.
    pub fn log(&self, msg: LogMsg) {
        self.logger.send(msg).unwrap()
    }
}

unsafe impl Sync for Server { }
