use std::{
    net::TcpStream,
    collections::HashMap,
    sync::RwLock,
};

pub enum PlayerCommand {
    Connect(String, TcpStream),
    Disconnect(String)
}

pub struct Players {
    players: RwLock<HashMap<String, TcpStream>>
}