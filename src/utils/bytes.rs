use crate::utils::any::Any;
use parking_lot::Mutex;
use std::{
    io::{Error as IoError, Write},
    ops::DerefMut,
    sync::Arc,
};

#[derive(Clone, Default)]
pub struct Bytes {
    value: Arc<Mutex<Vec<u8>>>,
}

impl Bytes {
    pub fn read(&self) -> Vec<u8> {
        self.value.lock().deref_mut().mem_take()
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
