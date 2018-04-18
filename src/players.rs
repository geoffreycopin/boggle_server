use super::{
    mock::StreamMock,
    server::ClientStream,
    errors::{ServerError, ServerError::*},
    game::GameCommand,
};

use std::{
    io::{BufRead, Write, Read},
    collections::{HashMap, HashSet},
    iter::FromIterator,
    sync::mpsc::{Sender, Receiver, channel},
};


pub enum PlayersCommand {
    Login(String, ClientStream)
}

pub struct Players<T: Write> {
    players: HashMap<String, T>,
    game: Sender<GameCommand>
}

impl<T: Write> Players<T> {
    pub fn new(game: Sender<GameCommand>) -> Self {
        Players { players: HashMap::new(), game }
    }

    pub fn login (&mut self, username: &str, mut stream: T) -> Result<(), ServerError> {
        if self.players.contains_key(username) {
            Err(ExistingUser{ username: username.to_string() })
        } else {
            self.register_user(username, stream);
            Ok(())
        }
    }

    fn register_user(&mut self, pseudo: &str, mut stream: T) {
        let message = format!("CONNECTE/{}/\n", pseudo);
        self.broadcast_message(&message);
        self.players.insert(pseudo.to_string(), stream);
    }

    fn broadcast_message(&mut self, message: &str) {
        for s in self.players.values_mut() {
            s.write(message.as_bytes());
        }
    }

    pub fn logout(&mut self, username: &str) -> Result<(), ServerError> {
        if self.players.contains_key(username) {
            self.remove_user(username);
            Ok(())
        } else {
            Err(NonExistingUser { username: username.to_string() })
        }
    }

    fn remove_user(&mut self, username: &str) {
        self.players.remove(username);
        self.broadcast_message(&format!("SORT/{}/", username));
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn login_ok() {
        let mut players = create_empty_players();
        players.login("newPlayer", StreamMock::new());
        let users = players.players;
        assert_eq!(users.keys().collect::<Vec<&String>>(), vec!["newPlayer"]);
    }

    #[test]
    fn login_existing_user_returns_error() {
        let mut players = create_empty_players();
        players.login("newPlayer", StreamMock::new());
        match players.login("newPlayer", StreamMock::new()) {
            Err(ExistingUser {..}) => (),
            _ => panic!("This call should return ServerError::ExistingUser")
        }
    }

    #[test]
    fn login_broadcast_to_others() {
        let (mut players, streams) = create_test_players();
        players.login("newUser", StreamMock::new());
        streams.iter().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert_eq!(last_line, "CONNECTE/newUser/")
        });
    }

    #[test]
    fn logout_ok() {
        let (mut players, streams) = create_test_players();
        players.logout("user2");
        let users = players.players;

        let actual = users.keys().map(|s| s.clone()).collect::<HashSet<String>>();

        let mut expected = HashSet::new();
        expected.insert("user3".to_string());
        expected.insert("user1".to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn logout_nonexisting_returns_error() {
        let (mut players, _) = create_test_players();
        match players.logout("user4") {
            Err(NonExistingUser {..}) => (),
            _ => panic!("This call should return ServerError::NonExistingUser")
        }
    }

    #[test]
    fn logout_broadcast_to_others() {
        let (mut players, _) = create_test_players();
        players.logout("user2");
        let users = players.players;
        users.values().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert_eq!(last_line, "SORT/user2/")
        })
    }

    fn create_test_players() -> (Players<StreamMock>, Vec<StreamMock>) {
        let mut players = create_empty_players();
        let streams = add_users(&mut players, &vec!["user1", "user2", "user3"]);
        (players, streams)
    }

    fn create_empty_players() {
        let (game_send, game_receive) = channel();
        Players::new(game_send);
    }

    fn add_users(players: &mut Players<StreamMock>, usernames: &[&str]) -> Vec<StreamMock> {
        let mut streams = Vec::new();
        for u in usernames {
            let s = StreamMock::new();
            players.login(u, s.clone());
            streams.push(s);
        }
        streams
    }
}