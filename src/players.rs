use super::{
    server::*,
    errors::{ServerError, ServerError::*},
};

use std::{
    io::Write,
    collections::HashMap,
};

pub struct Players<T: Write> {
    players: HashMap<String, T>
}

impl<T: Write> Players<T> {
    pub fn new() -> Self {
        Players { players: HashMap::new()}
    }

    pub fn login (&mut self, name: &str, stream: T) -> Result<(), ServerError> {
        if self.players.contains_key(name) {
            Err(ServerError::existing_user(name.to_string()))
        } else {
            self.register_user(name, stream);
            Ok(())
        }
    }

    pub fn users(&self) -> Vec<String> {
        self.players.keys()
            .map(|key| key.to_string())
            .collect()
    }

    fn register_user(&mut self, pseudo: &str, stream: T) {
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
            Err(ServerError::non_existing_user(username.to_string()))
        }
    }

    fn remove_user(&mut self, username: &str) {
        self.players.remove(username);
        self.broadcast_message(&format!("DECONNEXION/{}/", username));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::mock::StreamMock;
    use std::collections::HashSet;

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
            assert_eq!(last_line, "DECONNEXION/user2/")
        })
    }

    #[test]
    fn users() {
        let (players, _) = create_test_players();
        assert_eq!(players.users().len(), 3);
        assert!(players.users().contains(&"user1".to_string()));
        assert!(players.users().contains(&"user2".to_string()));
        assert!(players.users().contains(&"user3".to_string()));
    }

    fn create_test_players() -> (Players<StreamMock>, Vec<StreamMock>) {
        let mut players = create_empty_players();
        let streams = add_users(&mut players, &vec!["user1", "user2", "user3"]);
        (players, streams)
    }

    fn create_empty_players<T: Write>() -> Players<T> {
        Players::new()
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