#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;
extern crate rand;
extern crate unidecode;
extern crate clap;
extern crate regex;

mod board;
mod cloneable_stream;
mod game;
mod players;
mod log;
mod mock;
mod errors;
mod server;
mod dict;

use std::{
    sync::{mpsc::channel, Arc},
    net::TcpListener,
    thread,
};

use clap::{App, Arg};

fn main() {
    App::new("boggle_server")
        .author("Geoffrey Copin - 3201050")
        .arg(Arg::with_name("port")
            .long("port")
            .value_name("PORT")
            .takes_value(true))
        .arg(Arg::with_name("tours")
            .long("tours")
            .value_name("NB TOURS")
            .takes_value(true))
        .arg(Arg::with_name("immediat")
            .long("immediat")
            .help("Active la verification immédiate"))
        .arg(Arg::with_name("grilles")
            .long("grilles")
            .value_name("grille1 grille2")
            .takes_value(true)
            .multiple(true))
        .get_matches();
    serve()
}

fn serve() {
    let (log_send, log_receive) = channel();
    let (server_send, server_receive) = channel();

    let server = server::Server::new(dict::LocalDict::new(), log_send.clone());
    thread::spawn(|| server::run(Arc::new(server), server_receive));

    thread::spawn(|| log::log(log_receive));

    let listener = TcpListener::bind("127.0.0.1:2018").unwrap();

    println!("Serving on port 2018...");

    for stream in listener.incoming() {
        let s = stream.unwrap();
        server_send.send(s).expect("Server shut down.");
    }
}
