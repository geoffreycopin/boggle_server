#[macro_use] extern crate failure;
extern crate rand;
extern crate unidecode;

mod board;
mod cloneable_stream;
mod game;
mod players;
mod log;
mod mock;
mod errors;
mod server;
mod dict;

use server::*;

use std::{
    sync::mpsc::channel,
    net::TcpListener,
    thread,
};

fn main() {
    serve()
}

fn serve() {
    let (log_send, log_receive) = channel();
    let (server_send, server_receive) = channel();

    let server = Server::new(dict::LocalDict::new(), log_send.clone());
    thread::spawn(|| server::run(server, server_receive));

    thread::spawn(|| log::log(log_receive));

    let listener = TcpListener::bind("127.0.0.1:2018").unwrap();

    println!("Serving on port 2018...");

    for stream in listener.incoming() {
        let s = stream.unwrap();
        server_send.send(s).expect("Server shut down.");
    }
}
