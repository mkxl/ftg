use crate::server::{Connect, Server};
use anyhow::Error as AnyhowError;
use crossterm::event::EventStream as CrosstermEventStream;
use futures::{SinkExt, StreamExt};
use mkutils::{Output, Screen, Socket, Utils};
use std::io::Write;

pub struct Client;

impl Client {
    const MOUSE_CAPTURE_ENABLED: bool = true;

    pub async fn connect(connect: Connect) -> Output<(), AnyhowError> {
        let mut screen = Screen::config()
            .with_mouse_capture(Self::MOUSE_CAPTURE_ENABLED)
            .build()?;
        let mut crossterm_event_stream = CrosstermEventStream::new();
        let mut socket = Socket::connect(Server::socket_filepath().as_std_path()).await?;

        socket.send(connect).await?;

        mkutils::loop_select! {
          crossterm_event_res_opt = crossterm_event_stream.next() => socket.send(crossterm_event_res_opt.check_next()??).await?,
          byte_str_output = socket.recv::<Vec<u8>>() => screen.writer_mut().write_all_then(&byte_str_output?)?.flush()?,
        }
    }
}
