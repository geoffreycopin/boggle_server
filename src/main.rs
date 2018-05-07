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

use board::Board;
use dict::LocalDict;
use players::Players;

use std::{
    sync::{mpsc::channel, Arc},
    net::TcpListener,
    thread,
    time::Duration,
};

use clap::{App, Arg};

fn main() {
    run();
}

fn run() {
    let conf = App::new("boggle_server")
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
        .arg(Arg::with_name("duree_tour")
            .long("duree_tour")
            .value_name("NB_SECS")
            .help("Définit la durée d'un tour.")
            .takes_value(true))
        .arg(Arg::with_name("duree_pause")
            .long("duree_pause")
            .value_name("NB_SECS")
            .help("Définit la durée de la pause entre deux tours.")
            .takes_value(true))
        .get_matches();

    let port = conf.value_of("port").unwrap_or("2018");
    let nb_tours = conf.value_of("tours").unwrap_or("10").parse::<u64>()
        .expect("tours doit être un nombre entier!");
    let immediat = conf.is_present("immediat");
    let grilles = conf.values_of("grilles").unwrap_or_default()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let duree_tour = conf.value_of("duree_tour").unwrap_or("180").parse::<u64>()
        .map_err(|e| eprintln!("{}", e))
        .map(|d| Duration::from_secs(d))
        .unwrap();
    let duree_pause = conf.value_of("duree_pause").unwrap_or("10").parse::<u64>()
        .map_err(|e| eprintln!("{}", e))
        .map(|d| Duration::from_secs(d))
        .unwrap();

    let board = Board::new(immediat, grilles);
    let dict = LocalDict::new();
    let players = Players::new();
    let game = game::Game::new(players, board, dict);

    let (log_send, log_receive) = channel();
    let (server_send, server_receive) = channel();

    let server = server::Server::new(log_send.clone())
        .with_game(game)
        .with_pause_duration(duree_pause)
        .with_turn_duration(duree_tour)
        .with_nb_turn(nb_tours);

    thread::spawn(|| server::run(Arc::new(server), server_receive));

    thread::spawn(|| log::log(log_receive));

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .expect("Cannot start listener!");

    println!("Démarrage sur le port {}...", port);

    for stream in listener.incoming() {
        let s = stream.unwrap();
        server_send.send(s).expect("Le server s'est arrété!");
    }
}
