use crate::editor::color_scheme::Spec;
use ansi_parser::{AnsiParser, Output};
use futures::{Sink, SinkExt};
use num_traits::Num;
use parking_lot::Mutex;
use poem::web::websocket::Message as PoemMessage;
use postcard::Error as PostcardError;
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Span, Text},
    widgets::Paragraph,
};
use ropey::Rope;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;
use serde_yaml::Error as SerdeYamlError;
use std::{
    borrow::Borrow,
    fmt::Display,
    fs::File,
    future::Future,
    hash::{DefaultHasher, Hash, Hasher},
    io::{BufReader, BufWriter, Error as IoError, Read, Write},
    iter::Once,
    ops::BitAnd,
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

    fn color<'a, T2, T1: Stylize<'a, T2>>(self, spec: &Spec) -> T2
    where
        Self: Stylize<'a, T1>,
    {
        self.fg(spec.fg).bg(spec.bg)
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

    fn deserialize_from_yaml<'a, T: Deserialize<'a>>(&'a self) -> Result<T, SerdeYamlError>
    where
        Self: AsRef<str>,
    {
        serde_yaml::from_str(self.as_ref())
    }

    // NOTE: [https://stackoverflow.com/questions/69051429/what-is-the-function-to-get-the-quotient-and-remainder-divmod-for-rust#comment122040171_69051429]
    fn divmod(self, other: Self) -> (Self, Self)
    where
        Self: Copy + Num,
    {
        (self / other, self % other)
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

    fn hashcode(&self) -> u64
    where
        Self: Hash,
    {
        let mut hasher = DefaultHasher::new();

        self.hash(&mut hasher);

        hasher.finish()
    }

    fn inode_id(self) -> Result<Ulid, IoError>
    where
        Self: AsRef<Path>,
    {
        self.as_ref().metadata()?.ino().convert::<u128>().convert::<Ulid>().ok()
    }

    fn into_inner<T>(self) -> T
    where
        Self: Into<Result<T, T>>,
    {
        match self.into() {
            Ok(value) | Err(value) => value,
        }
    }

    fn is_even(&self) -> bool
    where
        Self: Copy + Num + BitAnd<Self, Output = Self>,
    {
        (*self) & Self::one() == Self::zero()
    }

    fn row_at(self, x: u16, y: u16) -> Rect
    where
        Self: Into<u16>,
    {
        Rect {
            x,
            y,
            width: self.into(),
            height: 1,
        }
    }

    fn mem_take(&mut self) -> Self
    where
        Self: Default,
    {
        std::mem::take(self)
    }

    fn mutex(self) -> Mutex<Self> {
        Mutex::new(self)
    }

    fn none<T>(&self) -> Option<T> {
        None
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

    fn paragraph<'a>(self) -> Paragraph<'a>
    where
        Self: Into<Text<'a>>,
    {
        Paragraph::new(self.into())
    }

    fn push_all_to(self, values: &mut Vec<Self::Item>)
    where
        Self: IntoIterator,
    {
        values.extend(self);
    }

    fn read_to_string(&self) -> Result<String, IoError>
    where
        Self: AsRef<Path>,
    {
        std::fs::read_to_string(self)
    }

    fn rect<T: Borrow<u16>>(&self) -> Rect
    where
        Self: Borrow<(T, T)>,
    {
        let (width, height) = self.borrow();

        Rect::new(0, 0, *width.borrow(), *height.borrow())
    }

    fn replace_with(&mut self, src: Self) -> Self {
        std::mem::replace(self, src)
    }

    fn rope(&self) -> Result<Rope, IoError>
    where
        Self: AsRef<Path>,
    {
        Rope::from_reader(self.open()?.buf_reader())
    }

    fn saturating_sub_from_top(self, dy: u16) -> Rect
    where
        Self: Into<Rect>,
    {
        let Rect { x, y, width, height } = self.into();

        Rect {
            x,
            y: y.saturating_add(dy),
            width,
            height: height.saturating_sub(dy),
        }
    }

    async fn select<F: Unpin + Future<Output = Self::Output>>(&mut self, other: F) -> Self::Output
    where
        Self: Future + Unpin,
    {
        futures::future::select(self, other).await.factor_first().0
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

    fn spans<'a>(&mut self, count: usize) -> impl Iterator<Item = Span<'a>>
    where
        Self: Iterator<Item = &'a str>,
    {
        self.take(count).map(Span::raw)
    }

    fn unit(self) {}

    fn warn<T, E: Display>(self) -> Option<T>
    where
        Self: Into<Result<T, E>>,
    {
        match self.into() {
            Ok(value) => value.some(),
            Err(error) => tracing::warn!(%error).none(),
        }
    }

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

    fn write_iter<S: AsRef<[u8]>, I: IntoIterator<Item = S>>(&mut self, items: I) -> Result<(), IoError>
    where
        Self: Write,
    {
        for item in items {
            self.write_all(item.as_ref())?;
        }

        self.flush()
    }
}

impl<T> Any for T {}
