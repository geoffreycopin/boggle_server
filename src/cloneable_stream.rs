use std::{
    io::{self, Read, Write},
    net::TcpStream
};

pub struct CloneableStream {
    stream: TcpStream
}

impl CloneableStream {
    pub fn new(stream: TcpStream) -> CloneableStream {
        CloneableStream { stream }
    }
}

impl Read for CloneableStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.stream.read(buf)
    }
}

impl Write for CloneableStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        self.stream.flush()
    }
}

impl Clone for CloneableStream {
    fn clone(&self) -> Self {
        CloneableStream::new(self.stream.try_clone().unwrap())
    }
}

