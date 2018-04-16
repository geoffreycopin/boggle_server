use super::{
    mock::StreamMock,
    errors::{ServerError, ServerError::*},
};

use std::{
    io::{Write, Read},
    collections::{HashMap, HashSet},
    iter::FromIterator,
    sync::RwLock,
};


pub struct Players<T: Write> {
    players: RwLock<HashMap<String, T>>
}

impl<T: Write> Players<T> {
    pub fn new() -> Self {
        Players { players: RwLock::new(HashMap::new()) }
    }

    pub fn login (&mut self, username: &str, mut stream: T) -> Result<(), ServerError> {
        let mut guard = self.players.write().unwrap();
        if guard.contains_key(username) {
            Err(ExistingUser{ username: username.to_string() })
        } else {
            Players::register_user(username, stream, &mut guard);
            Ok(())
        }
    }

    fn register_user(pseudo: &str, mut stream: T, users: &mut HashMap<String, T>) {
        let message = format!("CONNECTE/{}/\n", pseudo);
        Players::broadcast_message(&message, users);
        users.insert(pseudo.to_string(), stream);
    }

    fn broadcast_message(message: &str, users: &mut HashMap<String, T>) {
        for s in users.values_mut() {
            s.write(message.as_bytes());
        }
    }

    pub fn logout(&mut self, username: &str) -> Result<(), ServerError> {
        let mut guard = self.players.write().unwrap();
        if guard.contains_key(username) {
            Players::remove_user(username, &mut guard);
            Ok(())
        } else {
            Err(NonExistingUser { username: username.to_string() })
        }
    }

    fn remove_user(username: &str, users: &mut HashMap<String, T>) {
        users.remove(username);
        Players::broadcast_message(&format!("SORT/{}/", username), users);
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn login_ok() {
        let mut players = Players::new();
        players.login("newPlayer", StreamMock::new());
        let users = players.players.into_inner().unwrap();
        assert_eq!(users.keys().collect::<Vec<&String>>(), vec!["newPlayer"]);
    }

    #[test]
    fn login_existing_user_returns_error() {
        let mut players = Players::new();
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
        let users = players.players.into_inner().unwrap();

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
        let users = players.players.into_inner().unwrap();
        users.values().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert_eq!(last_line, "SORT/user2/")
        })
    }

    fn create_test_players() -> (Players<StreamMock>, Vec<StreamMock>) {
        let mut players = Players::new();
        let streams = add_users(&mut players, &vec!["user1", "user2", "user3"]);
        (players, streams)
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