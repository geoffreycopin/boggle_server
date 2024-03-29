use super::{
    board::Board,
    errors::ServerError,
    players::Players,
    dict::Dict
};

use std::{
    io::{Write},
    sync::{RwLock, Mutex, Condvar},
};

pub struct Game<T: Write + Clone> {
    players: RwLock<Players<T>>,
    board: RwLock<Board>,
    dict: RwLock<Box<Dict>>,
    turn_running: Mutex<bool>,
    turn_cond: Condvar,
}

impl<T: Write + Clone> Game<T> {
    pub fn new<U: Dict + 'static>(players: Players<T>, board: Board, dict: U) -> Self {
        Game {
            players: RwLock::new(players),
            board: RwLock::new(board),
            dict: RwLock::new(Box::new(dict)),
            turn_running: Mutex::new(false),
            turn_cond: Condvar::new(),
        }
    }

    /// Enregistre l'utilisateur `username`.
    /// En cas de succès, le message de bienvenue sera écrit sur le flux `steam`.
    /// REnvoie une erreur si l'utilisateur éxiste déjà.
    pub fn login(&self, username: &str, mut stream: T) -> Result<(), ServerError> {
        self.board.write().unwrap().add_user(username);
        let mut guard = self.players.write().unwrap();
        let res = guard.login(username, stream.clone());
        drop(guard);
        res.map(|_| self.welcome(&mut stream))
    }

    /// Ecrit le message de bienvenue sur le flux `stream`.
    fn welcome(&self, stream: &mut T) {
        let mut running = self.turn_running.lock().unwrap();
        while ! *running {
            running = self.turn_cond.wait(running).unwrap();
        }
        let board = self.board.read().unwrap();
        let welcome_str = board.welcome_str();
        stream.write(welcome_str.as_bytes()).unwrap();
    }

    /// Supprime l'utilisateur `username`.
    /// Renvoie une erreur si l'utilisateur n'éxistait pas.
    pub fn logout(&self, username: &str) -> Result<(), ServerError> {
        let res = self.players.write().unwrap().logout(username);
        self.board.write().unwrap().remove_user(username);
        res
    }

    /// Démarre une session de jeu.
    pub fn start_session(&self) {
        let mut players = self.players.write().unwrap();
        players.broadcast_message("SESSION/\n");
    }

    /// Met fin à la sessionde jeu courante.
    pub fn end_session(&self) {
        let msg = self.board.write().unwrap().scores_str();
        let mut players = self.players.write().unwrap();
        players.broadcast_message(&format!("VAINQUEUR/{}/\n", msg));
        drop(players);

        self.board.write().unwrap().reset();
    }

    /// Démarre un nouveau tour.
    pub fn new_turn(&self) {
        let mut board = self.board.write().unwrap();
        board.new_turn();
        let grid = board.grid_str();
        let msg = format!("TOUR/{}/\n", grid);
        drop(board);

        let mut players = self.players.write().unwrap();
        players.broadcast_message(&msg);

        let mut running = self.turn_running.lock().unwrap();
        *running = true;
        self.turn_cond.notify_all();
    }

    /// Met fin au tour courant.
    pub fn end_turn(&self) {
        *self.turn_running.lock().unwrap() = false;
        let message = self.board.write().unwrap().turn_scores();
        let mut players = self.players.write().unwrap();
        players.broadcast_message("RFIN/\n");
        players.broadcast_message(&message);
    }

    /// Analyse le mot `word`de trajectoire `trajectory` soumis par le joueur `username`.
    pub fn found(&self, username: &str, word: &str, trajectory: &str)
        -> Result<bool, ServerError>
    {
        let word = word.to_lowercase();
        self.check_exists(&word)?;
        let mut board = self.board.write().unwrap();
        board.submit_word(username, &word, trajectory)
    }

    pub fn check_exists(&self, word: &str) -> Result<(), ServerError> {
        let dict = self.dict.read().unwrap();
        if dict.contains(word) {
            Ok(())
        } else {
            Err(ServerError::non_existing_word(word))
        }
    }

    pub fn is_connected(&self, username: &str) -> bool {
        let players = self.players.read().unwrap();
        players.is_connected(username)
    }

    pub fn chat(&self, send: &str, recv: &str, msg: &str) -> Result<(), ServerError> {
        self.players.write().unwrap().chat(send, recv, msg)
    }

    pub fn chat_all(&self, msg: &str) -> Result<(), ServerError> {
        let message = format!("RECEPTION/{}/\n", msg);
        self.players.write().unwrap().broadcast_message(&message);
        Ok(())
    }
}


#[cfg(test)]
pub mod test {
    use super::*;
    use super::super::{
        mock::StreamMock,
        board::board::test::create_test_board,
        dict::LocalDict,
        players::test::create_test_players,
    };

    #[test]
    fn login_welcomes_user() {
        let game: Game<StreamMock> = create_test_game();
        let user1_stream = StreamMock::new();
        game.login("user1", user1_stream.clone()).unwrap();
        assert_eq!(user1_stream.to_string(),
                   "BIENVENUE/LIDAREJULTNEATNG/1*user1*0/\n")
    }

    #[test]
    fn new_turn_is_broadcasted() {
        let mut game: Game<StreamMock> = create_test_game();
        let (players, streams) = create_test_players();
        game.players = RwLock::new(players);
        game.new_turn();
        streams.iter().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert!(last_line.starts_with("TOUR/"))
        })
    }

    #[test]
    fn end_session_is_broadcasted() {
        let mut game: Game<StreamMock> = create_test_game();
        let (players, streams) = create_test_players();
        game.players = RwLock::new(players);
        game.end_session();
        streams.iter().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert!(last_line.starts_with("VAINQUEUR/"))
        })
    }

    #[test]
    fn found() {
        let game: Game<StreamMock> = create_test_game();
        game.login("user1", StreamMock::new()).unwrap();
        let result = game.found("user1", "ILE", "A2A1B2");
        assert!(result.is_ok())
    }

    #[test]
    fn found_already_played() {
        let game: Game<StreamMock> = create_test_game();
        game.login("user1", StreamMock::new()).unwrap();
        game.found("user1", "ILE", "A2A1B2").unwrap();
        match game.found("user1", "ILE", "A2A1B2") {
            Err(ServerError::AlreadyPlayed {..}) => (),
            _ => panic!("{} has already been played !", "ILE")
        }
    }

    #[test]
    fn found_non_existing() {
        let game: Game<StreamMock> = create_test_game();
        match game.found("user1", "lid", "A1A2A3") {
            Err(ServerError::NonExistingWord {..}) => (),
            _ => panic!("\"{}\" doesn't exist !", "lid")
        }
    }

    #[test]
    fn chat_all() {
        let mut game: Game<StreamMock> = create_test_game();
        let (players, streams) = create_test_players();
        game.players = RwLock::new(players);
        game.chat_all("test").unwrap();
        streams.iter().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert_eq!(last_line, "RECEPTION/test/")
        })
    }

    pub fn create_test_game<T: Write + Clone>() -> Game<T> {
        let board = create_test_board();
        let players = Players::new();
        let dict = LocalDict::from_dictionary("dico_test.txt");
        let game = Game::new(players, board, dict);
        *game.turn_running.lock().unwrap() = true;
        game
    }
}