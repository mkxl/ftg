use parking_lot::Mutex;
use std::{
    io::{Error as IoError, Write},
    sync::Arc,
};

#[derive(Clone, Default)]
pub struct Bytes {
    value: Arc<Mutex<Vec<u8>>>,
}

impl Bytes {
    pub fn take(&self) -> Vec<u8> {
        std::mem::take(&mut *self.value.lock())
    }
}

impl Write for Bytes {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        self.value.lock().write(buf)
    }

    fn flush(&mut self) -> Result<(), IoError> {
        self.value.lock().flush()
    }
}
