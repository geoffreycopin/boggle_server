pub mod server;

pub use self::server::{Server, Request};

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
    sync::{RwLock, Mutex,mpsc::{Sender, Receiver, channel}, Arc},
    thread::{self, JoinHandle},
    marker::Sync,
    net::{TcpStream, TcpListener, Shutdown},
    mem::replace,
    time::Duration,
};

pub fn run(server: Arc<Server>, streams: Receiver<TcpStream>) {
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
    println!("Reading next requests !");
    for req in reader.lines() {
        req.map(|r| {
            server.handle_client_request(&r, &username, writer.clone())
        });
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
        &"ENVOI" => parse_envoi(&components),
        &"PENVOI" => parse_penvoi(&components),
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

fn parse_envoi(components: &[&str]) -> Result<Request, ()> {
    let message = components.get(1).ok_or(())?;
    Ok(Request::ChatAll(message.to_string()))
}

fn parse_penvoi(components: &[&str]) -> Result<Request, ()> {
    let user = components.get(1).ok_or(())?;
    let message = components.get(2).ok_or(())?;
    Ok(Request::Chat(user.to_string(), message.to_string()))
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