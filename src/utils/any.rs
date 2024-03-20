use crate::utils::lock::Lock;
use futures::{Sink, SinkExt};
use poem::web::websocket::Message as PoemMessage;
use postcard::Error as PostcardError;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;
use std::{
    fmt::Display,
    fs::File,
    io::{BufReader, BufWriter, Error as IoError, Read, Write},
    path::Path,
};

pub trait Any: Sized {
    fn binary_message(self) -> PoemMessage
    where
        Self: Into<Vec<u8>>,
    {
        PoemMessage::binary(self.into())
    }

    fn buf_reader(self) -> BufReader<Self>
    where
        Self: Read,
    {
        BufReader::new(self)
    }

    fn buf_writer(self) -> BufWriter<Self>
    where
        Self: Write,
    {
        BufWriter::new(self)
    }

    fn convert<T: From<Self>>(self) -> T {
        self.into()
    }

    fn create(&self) -> Result<File, IoError>
    where
        Self: AsRef<Path>,
    {
        File::create(self)
    }

    fn decode<'a, T: Deserialize<'a>>(&'a self) -> Result<T, PostcardError>
    where
        Self: AsRef<[u8]>,
    {
        postcard::from_bytes(self.as_ref())
    }

    fn deserialize<'a, T: Deserialize<'a>>(&'a self) -> Result<T, SerdeJsonError>
    where
        Self: AsRef<str>,
    {
        serde_json::from_str(self.as_ref())
    }

    fn encode(&self) -> Result<Vec<u8>, PostcardError>
    where
        Self: Serialize,
    {
        postcard::to_stdvec(self)
    }

    fn error<T, E: Display>(self) -> Option<T>
    where
        Self: Into<Result<T, E>>,
    {
        match self.into() {
            Ok(ok) => ok.some(),
            Err(error) => tracing::error!(%error).with(None),
        }
    }

    fn locked(self) -> Lock<Self> {
        Lock::new(self)
    }

    fn ok<E>(self) -> Result<Self, E> {
        Ok(self)
    }

    fn open(&self) -> Result<File, IoError>
    where
        Self: AsRef<Path>,
    {
        File::open(self)
    }

    fn rope(&self) -> Result<Rope, IoError>
    where
        Self: AsRef<Path>,
    {
        Rope::from_reader(self.open()?.buf_reader())
    }

    async fn send_to<S: Unpin + Sink<Self>>(self, mut sink: S) -> Result<(), S::Error> {
        sink.send(self).await?.ok()
    }

    fn serialize(&self) -> Result<String, SerdeJsonError>
    where
        Self: Serialize,
    {
        serde_json::to_string(self)
    }

    fn some(self) -> Option<Self> {
        Some(self)
    }

    fn with<T>(&self, value: T) -> T {
        value
    }
}

impl<T> Any for T {}
