use super::errors::ServerError;

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
            Err(ServerError::existing_user(name))
        } else {
            self.register_user(name, stream);
            Ok(())
        }
    }

    pub fn is_connected(&self, username: &str) -> bool {
        self.players.contains_key(username)
    }

    fn register_user(&mut self, pseudo: &str, stream: T) {
        let message = format!("CONNECTE/{}/\n", pseudo);
        self.broadcast_message(&message);
        self.players.insert(pseudo.to_string(), stream);
    }

    pub fn broadcast_message(&mut self, message: &str) {
        for s in self.players.values_mut() {
            if let Err(e) = s.write(message.as_bytes()) {
                eprintln!("Error while broadcastin message: {}", e)
            }
        }
    }

    pub fn chat(&mut self, send: &str, recv: &str, msg: &str) -> Result<(), ServerError> {
        if ! self.players.contains_key(send) {
            return Err(ServerError::invalid_chat(send, recv, msg,
                                                 ServerError::non_existing_user(send)))
        }
        if ! self.players.contains_key(recv) {
            return Err(ServerError::invalid_chat(send, recv, msg,
                                                 ServerError::non_existing_user(recv)))
        }
        let stream = self.players.get_mut(recv).unwrap();
        stream.write(format!("PRECEPTION/{}/{}/\n", msg, send).as_bytes()).unwrap();
        Ok(())
    }

    pub fn logout(&mut self, username: &str) -> Result<(), ServerError> {
        if self.players.contains_key(username) {
            self.remove_user(username);
            Ok(())
        } else {
            Err(ServerError::non_existing_user(username))
        }
    }

    fn remove_user(&mut self, username: &str) {
        self.players.remove(username);
        self.broadcast_message(&format!("DECONNEXION/{}/", username));
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use super::super::errors::ServerError::*;
    use super::super::mock::StreamMock;
    use std::collections::HashSet;

    #[test]
    fn login_ok() {
        let mut players = create_empty_players();
        players.login("newPlayer", StreamMock::new()).unwrap();
        let users = players.players;
        assert_eq!(users.keys().collect::<Vec<&String>>(), vec!["newPlayer"]);
    }

    #[test]
    fn login_existing_user_returns_error() {
        let mut players = create_empty_players();
        players.login("newPlayer", StreamMock::new()).unwrap();
        match players.login("newPlayer", StreamMock::new()) {
            Err(ExistingUser {..}) => (),
            _ => panic!("This call should return ServerError::ExistingUser")
        }
    }

    #[test]
    fn login_broadcast_to_others() {
        let (mut players, streams) = create_test_players();
        players.login("newUser", StreamMock::new()).unwrap();
        streams.iter().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert_eq!(last_line, "CONNECTE/newUser/")
        });
    }

    #[test]
    fn logout_ok() {
        let (mut players, _) = create_test_players();
        players.logout("user2").unwrap();
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
        players.logout("user2").unwrap();
        let users = players.players;
        users.values().for_each(|s| {
            let last_line = s.to_string().lines().last().unwrap().to_owned();
            assert_eq!(last_line, "DECONNEXION/user2/")
        })
    }

    #[test]
    fn chat_ok() {
        let (mut players, _) = create_test_players();
        players.chat("user1", "user2", "Tu vas perdre !").unwrap();
        let user2_stream = players.players.get("user2").unwrap();
        let last_line = user2_stream.to_string().lines().last().unwrap().to_owned();
        assert_eq!(last_line, "PRECEPTION/Tu vas perdre !/user1/")
    }

    #[test]
    fn chat_nonexisting_sender() {
        let (mut players, _) = create_test_players();
        match players.chat("user5", "user2", "Tu vas perdre !") {
            Err(ServerError::InvalidChat {..}) => (),
            _ => panic!("This call should return an error !")
        }
    }

    #[test]
    fn chat_nonexisting_receiver() {
        let (mut players, _) = create_test_players();
        match players.chat("user2", "user5", "Tu vas perdre !") {
            Err(ServerError::InvalidChat {..}) => (),
            _ => panic!("This call should return an error !")
        }
    }

    pub fn create_test_players() -> (Players<StreamMock>, Vec<StreamMock>) {
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
            players.login(u, s.clone()).unwrap();
            streams.push(s);
        }
        streams
    }
}