use super::*;
use std::io::prelude::*;

pub enum Request {
    Login(String),
    Logout(String),
    Found(String, String)
}

pub struct Server {
    game: Game<CloneableWriter>,
    logger: Sender<LogMsg>,
    pub running_session: Mutex<Option<JoinHandle<()>>>,
}

impl Server {
    pub fn new<U: Dict  +'static>(dict: U, logger: Sender<LogMsg>) -> Server {
        let players = Players::new();
        let board = Board::new();
        let game = Game::new(players, board, dict);
        Server { game, logger, running_session: Mutex::new(None) }
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
            }
        });
        result.map_err(|e| {
            self.log(LogMsg::err(e))
        });
    }

    pub fn login(&self, username: &str, mut writer: CloneableWriter) -> Result<(), ServerError> {
        self.game.login(username, writer.clone())
            .map(|_|  self.log(LogMsg::login(username)))
            .map_err(|e| { writer.shutdown(); e })
    }

    pub fn logout(&self, username: &str, writer: CloneableWriter) -> Result<(), ServerError> {
        self.game.logout(username).map(|_| {
            writer.shutdown();
            self.log(LogMsg::Logout(username.to_string()))
        })
    }

    pub fn found(&self, username: &str, writer: &mut CloneableWriter, word: &str, trajectory: &str)
             -> Result<(), ServerError>
    {
        self.game.found(username, word, trajectory)
            .map(|_| {
                writer.write(format!("MVALIDE/{}/\n", word).as_bytes());
                self.log(LogMsg::Accepted(username.to_string(), word.to_string()))
            })
            .map_err(|e| {
                writer.write(format!("MINVALIDE/{}/\n", e).as_bytes());
                e
            })
    }

    pub fn remove_user_if_connected(&self, username: &str) {
        if self.game.is_connected(username) {
            self.game.logout(username)
                .map(|_| self.log(LogMsg::Logout(username.to_string())));
        }
    }

    pub fn log(&self, msg: LogMsg) {
        self.logger.send(msg).unwrap()
    }
}

unsafe impl Sync for Server { }

#[cfg(test)]
mod test {
    use super::*;
    use game::test::create_test_game;

    const ADDRESS: &str = "127.0.0.1:2018";

    /*#[test]
    fn welcome() {
        let (log_send, log_receive) = channel();
        let server = start_test_server(log_send);
        server.start_game_session();
        let mut stream = TcpStream::connect(ADDRESS).unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        stream.write("CONNEXION/Geoffrey/\n".as_bytes());

        let mut resp = String::new();

        reader.read_line(&mut resp);
        assert_eq!(&resp, "TOUR/LIDAREJULTNEATNG/\n");

        reader.read_line(&mut resp);
        assert_eq!(&resp, "BIENVENUE/LIDAREJULTNEATNG/1*Geoffrey*0/\n");

        let commands = vec!["CONNEXION/Geoffrey/\n".to_string()];
        let responses = vec!["BIENVENUE/LIDAREJULTNEATNG/1*Geoffrey*0/".to_string()];
        test_sequence(commands, responses);
    }*/

    /*fn test_sequence(commands: Vec<String>, responses: Vec<String>) {
        let (log_send, log_receive) = channel();
        let server = start_test_server(log_send);
        let mut stream = TcpStream::connect(ADDRESS).unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        commands.iter().zip(responses).for_each(|(c, r)| {
            eprintln!("c = {:#?}", c);
            stream.write(c.as_bytes());
            let resp = reader.lines().last().unwrap().unwrap();
            eprintln!("resp = {:#?}", resp);
            assert_eq!(resp, r);
        });


        stream.shutdown(Shutdown::Both);
        println!("DONE");
    }*/

    fn start_test_server(log_send: Sender<LogMsg>) -> Arc<Server> {
        let mut server = Server::new(LocalDict::new(), log_send);
        server.game = create_test_game();
        let server = Arc::new(server);
        let sv = server.clone();
        let listener = TcpListener::bind("127.0.0.1:2018").unwrap();
        thread::spawn(move || {
            for stream in listener.incoming() {
                let s = stream.unwrap();
                let server_copy = sv.clone();
                thread::spawn(move || start_connection(server_copy, s));
                return;
            }
        });

        server
    }


}