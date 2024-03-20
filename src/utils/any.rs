use crate::{editor::editor::Event, error::Error, utils::lock::Lock};
use futures::{Sink, SinkExt};
use poem::web::websocket::Message as PoemMessage;
use postcard::Error as PostcardError;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::File,
    io::{BufWriter, Error as IoError, Write},
    path::Path,
};
use tokio_tungstenite::tungstenite::{Error as TungsteniteError, Message as TungsteniteMessage};

pub trait Any: Sized {
    fn binary_message(self) -> PoemMessage
    where
        Self: Into<Vec<u8>>,
    {
        PoemMessage::binary(self.into())
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

    async fn send_event_to<S: Unpin + Sink<TungsteniteMessage, Error = TungsteniteError>>(
        self,
        sink: S,
    ) -> Result<(), Error>
    where
        Event: From<Self>,
    {
        self.convert::<Event>()
            .encode()?
            .convert::<TungsteniteMessage>()
            .send_to(sink)
            .await?
            .ok()
    }

    async fn send_to<S: Unpin + Sink<Self>>(self, mut sink: S) -> Result<(), S::Error> {
        sink.send(self).await?.ok()
    }

    fn some(self) -> Option<Self> {
        Some(self)
    }

    fn with<T>(&self, value: T) -> T {
        value
    }
}

impl<T> Any for T {}
