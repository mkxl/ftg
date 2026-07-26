use crate::{
    cli_args::LoggingArgs,
    server::{Connect, Server},
};
use anyhow::Error as AnyhowError;
use camino::Utf8Path;
use crossterm::event::EventStream as CrosstermEventStream;
use futures::{SinkExt, StreamExt};
use mkutils::{Output, Screen, Socket, Utils};
use std::{
    io::{Error as IoError, Write},
    time::Duration,
};

pub struct Client;

impl Client {
    const MOUSE_CAPTURE_ENABLED: bool = true;
    const BRACKETED_PASTE_ENABLED: bool = true;
    const SERVER_WARMUP_DURATION: Duration = Duration::from_millis(25);
    const LINE_WRAP_DISABLED: bool = false;

    async fn run(connect: Connect, mut socket: Socket) -> Output<(), AnyhowError> {
        let mut screen = Screen::config()
            .with_mouse_capture(Self::MOUSE_CAPTURE_ENABLED)
            .with_bracketed_paste(Self::BRACKETED_PASTE_ENABLED)
            .with_line_wrap_disabled(Self::LINE_WRAP_DISABLED)
            .build()?;
        let mut crossterm_event_stream = CrosstermEventStream::new();

        socket.send(connect).await?;

        mkutils::loop_select! {
          crossterm_event_res_opt = crossterm_event_stream.next() => socket.send(crossterm_event_res_opt.check_next()??).await?,
          bytes_output = socket.recv::<Vec<u8>>() => screen.stdout_mut().write_all_then(&bytes_output?)?.flush()?,
        }
    }

    async fn connect_to_socket() -> Result<Socket, IoError> {
        Socket::connect(Server::socket_filepath().as_std_path()).await
    }

    async fn wait_then_connect_then_run(connect: Connect) -> Output<(), AnyhowError> {
        Self::SERVER_WARMUP_DURATION.sleep().await;

        Self::run(connect, Self::connect_to_socket().await?).await
    }

    pub async fn connect(
        connect: Connect,
        config_filepath: Option<&Utf8Path>,
        logging_args: &LoggingArgs,
    ) -> Output<(), AnyhowError> {
        // NOTE:
        // - try connecting to the socket and then running
        // - if that fails, launch the server, wait, connect, and then run
        if let Ok(socket) = Self::connect_to_socket().await {
            return Self::run(connect, socket).await;
        }

        tokio::select! {
            unit_res = Server::serve(config_filepath, logging_args) => unit_res?.output_ok(),
            unit_output = Self::wait_then_connect_then_run(connect) => unit_output,
        }
    }
}
