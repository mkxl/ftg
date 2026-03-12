use crate::server::{Connect, Server};
use anyhow::Error as AnyhowError;
use camino::Utf8PathBuf;
use crossterm::event::EventStream as CrosstermEventStream;
use futures::{SinkExt, StreamExt};
use mkutils::{Output, Screen, Socket, Utils};
use std::{
    io::{Error as IoError, Write},
    time::Duration,
};
use tokio::task::JoinHandle;

pub struct Client;

impl Client {
    const MOUSE_CAPTURE_ENABLED: bool = true;
    const SERVER_WARMUP_DURATION: Duration = Duration::from_millis(25);

    async fn socket() -> Result<Socket, IoError> {
        Socket::connect(Server::socket_filepath().as_std_path()).await
    }

    async fn serve_and_connect(
        config_filepath: Option<Utf8PathBuf>,
    ) -> Result<(Socket, Option<JoinHandle<Result<(), AnyhowError>>>), IoError> {
        if let Ok(socket) = Self::socket().await {
            return socket.pair(None).ok();
        }

        // NOTE: converting a future to a task requires the future contain only static references
        // effectively requiring that [config_filepath] be [Option<Utf8PathBuf>] rather than
        // [Option<&Utf8Path>]
        let serve_task_opt = Server::serve(config_filepath).spawn_task().some();

        Self::SERVER_WARMUP_DURATION.sleep().await;

        Self::socket().await?.pair(serve_task_opt).ok()
    }

    pub async fn connect(connect: Connect, config_filepath: Option<Utf8PathBuf>) -> Output<(), AnyhowError> {
        let mut screen = Screen::config()
            .with_mouse_capture(Self::MOUSE_CAPTURE_ENABLED)
            .build()?;
        let mut crossterm_event_stream = CrosstermEventStream::new();
        let (mut socket, mut serve_task_opt) = Self::serve_and_connect(config_filepath).await?;

        socket.send(connect).await?;

        mkutils::loop_select! {
          crossterm_event_res_opt = crossterm_event_stream.next() => socket.send(crossterm_event_res_opt.check_next()??).await?,
          bytes_output = socket.recv::<Vec<u8>>() => screen.writer_mut().write_all_then(&bytes_output?)?.flush()?,
          unit_res_res = serve_task_opt.unwrap_or_pending_then_wait() => unit_res_res??,
        }
    }
}
