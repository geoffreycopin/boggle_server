#[macro_use] extern crate failure;
extern crate rand;

mod game;
mod players;
mod log;
mod mock;
mod errors;
mod request_handler;

use request_handler::RequestHandler;
use log::start_logging;

use std::{
    cell::RefCell,
    sync::Arc,
    net::{TcpListener, TcpStream},
};

fn main() {
    serve()
}

fn serve() {
    let logger = start_logging();
    let handler = Arc::new(RefCell::new(RequestHandler::new(logger)));
    let listener = TcpListener::bind("127.0.0.1:2018").unwrap();

    println!("Serving on port 2018...");

    for stream in listener.incoming() {
        handle_client(stream.unwrap(), handler.clone())
    }
}

fn handle_client(stream: TcpStream, handler: Arc<RefCell<RequestHandler<TcpStream>>>) {
    println!("Connection established !")
}
