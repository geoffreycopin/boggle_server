use std::{
    io::{self, Read, Write},
    net::{TcpStream, Shutdown}
};

pub struct CloneableWriter {
    stream: TcpStream
}

impl CloneableWriter {
    pub fn new(stream: TcpStream) -> CloneableWriter {
        CloneableWriter { stream }
    }

    pub fn shutdown(self) {
        self.stream.shutdown(Shutdown::Both);
    }
}

impl Write for CloneableWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        self.stream.flush()
    }
}

impl Clone for CloneableWriter {
    fn clone(&self) -> Self {
        CloneableWriter::new(self.stream.try_clone().unwrap())
    }
}

