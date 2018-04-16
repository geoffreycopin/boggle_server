use std::{
    cell::RefCell,
    io::{Write, Error},
    rc::Rc,
};

#[derive(Debug, Clone)]
pub struct StreamMock {
    data: Rc<RefCell<Vec<u8>>>
}

impl StreamMock {
    pub fn new() -> StreamMock {
        StreamMock { data: Rc::new(RefCell::new(Vec::new())) }
    }

    pub fn to_string(&self) -> String {
        String::from_utf8(self.data.borrow().to_vec()).unwrap()
    }
}

impl Write for StreamMock {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.data.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}