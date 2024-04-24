use ansi_parser::{AnsiParser, Output};
use futures::{Sink, SinkExt};
use parking_lot::Mutex;
use poem::web::websocket::Message as PoemMessage;
use postcard::Error as PostcardError;
use ratatui::layout::Rect;
use ropey::Rope;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;
use serde_yaml::Error as SerdeYamlError;
use std::{
    fmt::Display,
    fs::File,
    io::{BufReader, BufWriter, Error as IoError, Read, Write},
    iter::Once,
    os::unix::fs::MetadataExt,
    path::Path,
    sync::Arc,
};
use ulid::Ulid;

pub trait Any: Sized {
    // NOTE: likely will be used in the future for debugging
    #[allow(dead_code)]
    fn ansi(&self) -> Option<Vec<Output>>
    where
        Self: AsRef<[u8]>,
    {
        let string = std::str::from_utf8(self.as_ref()).ok()?;
        let outputs = string.ansi_parse().collect::<Vec<_>>();

        outputs.some()
    }

    fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

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

    fn deserialize_from_json<'a, T: Deserialize<'a>>(&'a self) -> Result<T, SerdeJsonError>
    where
        Self: AsRef<str>,
    {
        serde_json::from_str(self.as_ref())
    }

    fn deserialize_from_yaml_reader<T: DeserializeOwned>(self) -> Result<T, SerdeYamlError>
    where
        Self: Read,
    {
        serde_yaml::from_reader(self)
    }

    fn encode(&self) -> Result<Vec<u8>, PostcardError>
    where
        Self: Serialize,
    {
        postcard::to_stdvec(self)
    }

    fn err<T>(self) -> Result<T, Self> {
        Err(self)
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

    fn inode_id(self) -> Result<Ulid, IoError>
    where
        Self: AsRef<Path>,
    {
        self.as_ref().metadata()?.ino().convert::<u128>().convert::<Ulid>().ok()
    }

    fn mutex(self) -> Mutex<Self> {
        Mutex::new(self)
    }

    fn ok<E>(self) -> Result<Self, E> {
        Ok(self)
    }

    fn once(self) -> Once<Self> {
        std::iter::once(self)
    }

    fn open(&self) -> Result<File, IoError>
    where
        Self: AsRef<Path>,
    {
        File::open(self)
    }

    fn rect(self) -> Rect
    where
        Self: Into<(u16, u16)>,
    {
        let (width, height) = self.into();

        Rect::new(0, 0, width, height)
    }

    fn rope(&self) -> Result<Rope, IoError>
    where
        Self: AsRef<Path>,
    {
        Rope::from_reader(self.open()?.buf_reader())
    }

    fn saturating_sub(self, dx: u16, dy: u16) -> Rect
    where
        Self: Into<Rect>,
    {
        let Rect { x, y, width, height } = self.into();

        Rect {
            x,
            y,
            width: width.saturating_sub(dx),
            height: height.saturating_sub(dy),
        }
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

    fn mem_take(&mut self) -> Self
    where
        Self: Default,
    {
        std::mem::take(self)
    }

    fn unit(self) {}

    fn with<T>(&self, value: T) -> T {
        value
    }

    fn write_all_and_flush(&mut self, bytes: &[u8]) -> Result<(), IoError>
    where
        Self: Write,
    {
        self.write_all(bytes)?;
        self.flush()?;

        ().ok()
    }
}

impl<T> Any for T {}
