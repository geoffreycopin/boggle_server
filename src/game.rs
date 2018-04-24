use super::{
    board::Board,
    errors::ServerError,
    players::Players,
    dict::Dict
};

use std::{
    io::{Write},
    sync::RwLock,
};

pub struct Game<T: Write + Clone> {
    players: RwLock<Players<T>>,
    board: RwLock<Board>,
    dict: RwLock<Box<Dict>>,
}

impl<T: Write + Clone> Game<T> {
    pub fn new<U: Dict + 'static>(players: Players<T>, board: Board, dict: U) -> Self {
        Game {
            players: RwLock::new(players),
            board: RwLock::new(board),
            dict: RwLock::new(Box::new(dict))
        }
    }

    pub fn login(&self, username: &str, mut stream: T) -> Result<(), ServerError> {
        let mut guard = self.players.write().unwrap();
        guard.login(username, stream.clone())
            .map(|_| self.welcome(&mut stream, &guard.users()))
    }

    fn welcome(&self, stream: &mut T, users: &[String]) {
        let board = self.board.read().unwrap();
        let welcome_str = board.welcome_str(&users);
        stream.write(welcome_str.as_bytes()).unwrap();
    }

    pub fn logout(&self, username: &str) -> Result<(), ServerError> {
        let mut players = self.players.write().unwrap();
        players.logout(username)
    }

    pub fn start_session(&self) {
        let mut players = self.players.write().unwrap();
        players.broadcast_message("SESSION/\n");
    }

    pub fn end_session(&self) {
        let board = self.board.read().unwrap();
        let mut players = self.players.write().unwrap();
        let msg = board.scores_str(&players.users());
        players.broadcast_message(&format!("VAINQUEUR/{}/", msg));
    }

    pub fn new_turn(&self) {
        let mut board = self.board.write().unwrap();
        board.new_turn();
        let msg = format!("TOUR/{}/\n", board.grid_str());
        let mut players = self.players.write().unwrap();
        players.broadcast_message(&msg);
    }

    pub fn found(&self, username: &str, stream: &mut T, word: &str, trajectory: &str)
        -> Result<(), ServerError>
    {
        let word = word.to_lowercase();
        self.check_already_played(&word)?;
        self.check_exists(&word)?;
        let mut board = self.board.write().unwrap();
        board.submit_word(username, &word, trajectory)
    }

    pub fn check_already_played(&self, word: &str) -> Result<(), ServerError> {
        let board = self.board.read().unwrap();
        if board.is_already_played(word) {
            Err(ServerError::already_played(word))
        } else {
            Ok(())
        }
    }

    pub fn check_exists(&self, word: &str) -> Result<(), ServerError> {
        let dict = self.dict.read().unwrap();
        if dict.contains(word) {
            Ok(())
        } else {
            Err(ServerError::non_existing_word(word))
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::{
        mock::StreamMock,
        board::board::test::create_test_board,
        dict::LocalDict,
        players::test::create_test_players,
    };

    #[test]
    fn login_welcomes_user() {
        let mut game = create_test_game();
        let user1_stream = StreamMock::new();
        game.login("user1", user1_stream.clone());
        assert_eq!(user1_stream.to_string(),
                   "BIENVENUE/LIDAREJULTNEATNG/1*user1*0/\n")
    }

    #[test]
    fn new_turn_is_broadcasted() {
        let mut game = create_test_game();
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
        let mut game = create_test_game();
        let (players, mut streams) = create_test_players();
        game.players = RwLock::new(players);
        game.end_session();
        streams.iter().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert!(last_line.starts_with("VAINQUEUR/"))
        })
    }

    #[test]
    fn found() {
        let mut game = create_test_game();
        let result = game.found("user1", "ILE", "A2A1B2");
        assert!(result.is_ok())
    }

    #[test]
    fn found_already_played() {
        let mut game = create_test_game();
        game.found("user1", "ILE", "A2A1B2");
        match game.found("user1", "ILE", "A2A1B2") {
            Err(ServerError::AlreadyPlayed {..}) => (),
            _ => panic!("{} has already been played !", "ILE")
        }
    }

    #[test]
    fn found_non_existing() {
        let mut game = create_test_game();
        match game.found("user1", "lid", "A1A2A3") {
            Err(ServerError::NonExistingWord {..}) => (),
            _ => panic!("\"{}\" doesn't exist !", "lid")
        }
    }

    fn create_test_game() -> Game<StreamMock> {
        let board = create_test_board();
        let players = Players::new();
        let dict = LocalDict::from_dictionary("dico_test.txt");
        Game::new(players, board, dict)
    }
}