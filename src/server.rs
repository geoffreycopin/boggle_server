use super::{
    log::*,
    board::*,
    players::*,
    game::Game,
    errors::ServerError,
    dict::{Dict, LocalDict},
    cloneable_stream::CloneableWriter,
};

use std::{
    io::{BufReader, BufRead, Write},
    sync::{RwLock, Mutex,mpsc::{Sender, Receiver}, Arc},
    thread::{self, JoinHandle},
    marker::Sync,
    net::{TcpStream, Shutdown},
    mem::replace,
    time::Duration,
};

pub type LogChan = Sender<LogMsg>;

pub enum Request {
    Login(String),
    Logout(String),
    Found(String, String)
}

pub struct Server {
    game: Game<CloneableWriter>,
    logger: Sender<LogMsg>,
    running_session: Mutex<Option<JoinHandle<()>>>,
}

impl Server {
    pub fn new<U: Dict  +'static>(dict: U, logger: Sender<LogMsg>) -> Server {
        let players = Players::new();
        let board = Board::new();
        let game = Game::new(players, board, dict);
        Server { game, logger, running_session: Mutex::new(None) }
    }

    fn start_game_session(&self) {
        self.game.start_session();
        println!("Start session");
    }

    fn end_game_session(&self) {
        self.game.end_session();
        println!("End Session");
    }

    fn new_game_turn(&self) {
        self.game.new_turn();
        println!("start turn");
    }

    fn end_game_turn(&self) {
        self.game.end_turn();
    }

    fn handle_client_request(&self, request: &str, username: &str, mut stream: CloneableWriter) {
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

    fn login(&self, username: &str, mut writer: CloneableWriter) -> Result<(), ServerError> {
        self.game.login(username, writer.clone())
            .map(|_| self.log(LogMsg::login(username)))
            .map_err(|e| { writer.shutdown(); e })
    }

    fn logout(&self, username: &str, writer: CloneableWriter) -> Result<(), ServerError> {
        self.game.logout(username).map(|_| {
            writer.shutdown();
            self.log(LogMsg::Logout(username.to_string()))
        })
    }

    fn found(&self, username: &str, writer: &mut CloneableWriter, word: &str, trajectory: &str)
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

pub fn run(server: Server, streams: Receiver<TcpStream>) {
    let server = Arc::new(server);

    for sock in streams {
        let s = server.clone();
        thread::spawn(move || {
            start_connection(s, sock);
        });
    }
}

fn start_connection(server: Arc<Server>, stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let writer = CloneableWriter::new(stream);
    let mut username = String::new();

    match connect(server.clone(), writer.clone(), &mut reader) {
        Ok(name) => username = name,
        Err(e) => { server.log(LogMsg::Error(e)); return; }
    }

    for req in reader.lines() {
        server.handle_client_request(&req.unwrap(), &username, writer.clone())
    }

    server.remove_user_if_connected(&username);
}

fn connect(server: Arc<Server>, stream: CloneableWriter, reader: &mut BufReader<TcpStream>)
           -> Result<String, ServerError>
{
    let mut session = server.running_session.lock().unwrap();
    if let None = *session {
        replace(&mut *session, Some(run_session(server.clone())));
    }

    let mut req = String::new();
    reader.read_line(&mut req).unwrap();

    match parse_request(&req) {
        Ok(Request::Login(username)) => {
            server.login(&username, stream)?;
            Ok(username)
        },
        _ => Err(ServerError::unauthorized_request(&req))
    }
}

fn parse_request(req: &str) -> Result<Request, ServerError> {
    let components: Vec<&str> = req.split("/").collect();
    let err = ServerError::bad_request(req);

    let request = match components.get(0).ok_or(err.clone())? {
        &"CONNEXION" => parse_connexion(&components),
        &"SORT" => parse_sort(&components),
        &"TROUVE" => parse_trouve(&components),
        _ => Err(())
    };

    request.map_err(|_| err)
}

fn parse_connexion(components: &[&str]) -> Result<Request, ()> {
    let username = components.get(1).ok_or(())?;
    Ok(Request::Login(username.to_string()))
}

fn parse_sort(components: &[&str]) -> Result<Request, ()> {
    let username = components.get(1).ok_or(())?;
    Ok(Request::Logout(username.to_string()))
}

fn parse_trouve(components: &[&str]) -> Result<Request, ()> {
    let word = components.get(1).ok_or(())?;
    let trajectory = components.get(2).ok_or(())?;
    Ok(Request::Found(word.to_string(), trajectory.to_string()))
}

fn run_session(server: Arc<Server>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            server.start_game_session();
            for _ in 0..10 {
                server.new_game_turn();
                thread::sleep(Duration::from_secs(180));
                server.end_game_turn();
                thread::sleep(Duration::from_secs(10));
            }
            server.end_game_session();
        }
    })
}