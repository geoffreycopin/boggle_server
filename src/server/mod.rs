pub mod server;

pub use self::server::{Server, Request};

use super::{
    log::*,
    board::*,
    players::*,
    game::Game,
    errors::ServerError,
    cloneable_stream::CloneableWriter,
};

use std::{
    io::{BufReader, BufRead},
    sync::{Mutex,mpsc::{Sender, Receiver}, Arc},
    thread::{self, JoinHandle},
    marker::Sync,
    net::TcpStream,
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
    let username;

    match connect(server.clone(), writer.clone(), &mut reader) {
        Ok(name) => username = name,
        Err(e) => { server.log(LogMsg::Error(e)); return; }
    }

    for req in reader.lines() {
        if let Ok(r) = req {
            server.handle_client_request(&r, &username, writer.clone())
        }
    }
    server.remove_user_if_connected(&username);
}

fn connect(server: Arc<Server>, stream: CloneableWriter, reader: &mut BufReader<TcpStream>)
           -> Result<String, ServerError>
{
    if server.nb_players() == 0 {
        start_session(server.clone(), server.nb_turn(), server.turn_duration(), server.pause_duration());
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

fn start_session(server: Arc<Server>, nb_turn: u64, turn: Duration, pause: Duration)
    -> JoinHandle<()>
{
    thread::spawn(move || {
        loop {
            server.start_game_session();
            for _ in 0..nb_turn {
                server.new_game_turn();
                thread::sleep(turn);
                server.end_game_turn();
                thread::sleep(pause);
                if server.nb_players() == 0 {
                    return;
                }
            }
            server.end_game_session();
        }
    })
}