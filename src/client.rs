use crate::{cli::ClientArgs, editor::editor::Config, error::Error, server::Server, utils::any::Any};
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture, EventStream as CrosstermEventStream},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use derive_more::From;
use futures::StreamExt;
use std::{
    io::{StdoutLock, Write},
    sync::Mutex,
};
use tokio_tungstenite::tungstenite::Message;
use url::Url;

#[derive(From)]
pub struct Client {
    stdout: StdoutLock<'static>,
    args: ClientArgs,
}

impl Client {
    fn new(args: ClientArgs) -> Result<Self, Error> {
        let stdout = std::io::stdout().lock();
        let mut client = Self { stdout, args };

        client.on_init()?;

        client.ok()
    }

    fn init_tracing(&self) -> Result<(), Error> {
        let Some(log_filepath) = self.args.log_filepath.as_ref() else {
            return ().ok();
        };
        let writer = log_filepath.create()?.buf_writer();
        let writer = Mutex::new(writer);

        // TODO: consider using tracing-appender for writing to a file
        tracing_subscriber::fmt().with_writer(writer).json().init();

        ().ok()
    }

    fn on_init(&mut self) -> Result<(), Error> {
        self.init_tracing()?;
        crossterm::terminal::enable_raw_mode()?;
        self.stdout
            .queue(EnterAlternateScreen)?
            .queue(EnableMouseCapture)?
            .queue(Hide)?
            .queue(Clear(ClearType::All))?
            .flush()?;

        ().ok()
    }

    fn on_drop(&mut self) -> Result<(), Error> {
        crossterm::terminal::disable_raw_mode()?;
        self.stdout
            .queue(LeaveAlternateScreen)?
            .queue(DisableMouseCapture)?
            .queue(Show)?
            .flush()?;

        ().ok()
    }

    fn config(client_args: &ClientArgs) -> Result<Config, Error> {
        let size = crossterm::terminal::size()?.some();

        Config { size }.ok()
    }

    pub async fn run(client_args: ClientArgs) -> Result<(), Error> {
        let mut client = Client::new(client_args)?;
        let mut crossterm_events = CrosstermEventStream::new();
        let (web_socket, _response) = tokio_tungstenite::connect_async(&client.args.server_address).await?;
        let (mut sink, mut stream) = web_socket.split();

        Self::config(&client.args)?.send_event_to(&mut sink).await?;

        let recv = async {
            while let Some(message_res) = stream.next().await {
                match message_res? {
                    Message::Binary(bytes) => client.stdout.write_all(&bytes)?,
                    Message::Close(_close) => std::todo!(),
                    ignored_message => tracing::warn!(?ignored_message),
                }
            }

            ().ok()
        };
        let send = async {
            while let Some(crossterm_event_res) = crossterm_events.next().await {
                // TODO: figure out why this doesn't work
                // - crossterm_event_res?.encode()?.into().send_to(&mut sink).await?;
                crossterm_event_res?.send_event_to(&mut sink).await?;
            }

            ().ok()
        };

        crate::utils::macros::select!(recv, send)
    }

    pub fn default_server_address() -> Url {
        std::format!(
            "ws://{host}:{port}",
            host = Server::DEFAULT_HOST,
            port = Server::DEFAULT_PORT,
        )
        .parse()
        .unwrap()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.on_drop().error();
    }
}
