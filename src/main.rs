#[macro_use] extern crate failure;
extern crate rand;

mod game;
mod players;
mod log;
mod mock;
mod errors;
mod server;

use server::*;
use players::Players;

use std::{
    cell::RefCell,
    sync::{RwLock, mpsc::{Receiver, Sender, channel}},
    net::{TcpListener, TcpStream},
    io::{prelude::*, BufReader},
    thread,
};

fn main() {
    serve()
}

fn serve() {
    let (log_send, log_receive) = channel();
    let (players_send, players_receive) = channel();
    let (game_send, game_receive) = channel();
    let (server_send, server_receive) = channel();

    let server = Server::new(log_send.clone(), game_send.clone(), players_send.clone());
    thread::spawn(|| server.start(server_receive));

    let players: Players<TcpStream> = Players::new(game_send.clone());
    thread::spawn(|| players::run(players, players_receive));

    thread::spawn(|| log::log(log_receive));

    let listener = TcpListener::bind("127.0.0.1:2018").unwrap();

    println!("Serving on port 2018...");

    for stream in listener.incoming() {
        let s = stream.unwrap();
        server_send.send(s);
    }
}
