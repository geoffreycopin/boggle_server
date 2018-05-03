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
        let res = guard.login(username, stream.clone());
        let users = guard.users();
        drop(guard);
        res.map(|_| self.welcome(&mut stream, &users))
    }

    fn welcome(&self, stream: &mut T, users: &[String]) {
        let board = self.board.read().unwrap();
        let welcome_str = board.welcome_str(&users);
        stream.write(welcome_str.as_bytes()).unwrap();
        println!("Wrote response !")
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
        let mut board = self.board.write().unwrap();
        let mut players = self.players.write().unwrap();

        let msg = board.scores_str(&players.users());
        players.broadcast_message(&format!("VAINQUEUR/{}/\n", msg));

        board.reset();
    }

    pub fn new_turn(&self) {
        let mut grid = String::new();
        {
            let mut board = self.board.write().unwrap();
            board.new_turn();
            grid = board.grid_str();
        }
        let msg = format!("TOUR/{}/\n", grid);
        let mut players = self.players.write().unwrap();
        players.broadcast_message(&msg);
    }

    pub fn end_turn(&self) {
        let users = self.players.read().unwrap().users();
        let message = self.board.read().unwrap().turn_scores(&users);
        let mut players = self.players.write().unwrap();
        players.broadcast_message("RFIN/\n");
        players.broadcast_message(&message);
    }

    pub fn found(&self, username: &str, word: &str, trajectory: &str)
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
        let mut game: Game<StreamMock> = create_test_game();
        let user1_stream = StreamMock::new();
        game.login("user1", user1_stream.clone());
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
        let mut game: Game<StreamMock> = create_test_game();
        let result = game.found("user1", "ILE", "A2A1B2");
        assert!(result.is_ok())
    }

    #[test]
    fn found_already_played() {
        let mut game: Game<StreamMock> = create_test_game();
        game.found("user1", "ILE", "A2A1B2");
        match game.found("user1", "ILE", "A2A1B2") {
            Err(ServerError::AlreadyPlayed {..}) => (),
            _ => panic!("{} has already been played !", "ILE")
        }
    }

    #[test]
    fn found_non_existing() {
        let mut game: Game<StreamMock> = create_test_game();
        match game.found("user1", "lid", "A1A2A3") {
            Err(ServerError::NonExistingWord {..}) => (),
            _ => panic!("\"{}\" doesn't exist !", "lid")
        }
    }

    #[test]
    fn chat_all() {
        let mut game: Game<StreamMock> = create_test_game();
        let (players, mut streams) = create_test_players();
        game.players = RwLock::new(players);
        game.chat_all("test");
        streams.iter().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert_eq!(last_line, "RECEPTION/test/")
        })
    }

    pub fn create_test_game<T: Write + Clone>() -> Game<T> {
        let board = create_test_board();
        let players = Players::new();
        let dict = LocalDict::from_dictionary("dico_test.txt");
        Game::new(players, board, dict)
    }
}