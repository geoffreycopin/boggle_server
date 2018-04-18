#[macro_use] extern crate failure;
extern crate rand;

mod game;
mod players;
mod log;
mod mock;
mod errors;
mod server;

use server::*;
use log::start_logging;

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

    let server = Server::new(log_send, game_send, players_send);
    thread::spawn(|| server.start(server_receive));

    let listener = TcpListener::bind("127.0.0.1:2018").unwrap();

    println!("Serving on port 2018...");

    for stream in listener.incoming() {
        let s = stream.unwrap();
        server_send.send(s);
    }
}
