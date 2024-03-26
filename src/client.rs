use crate::{cli::ClientArgs, editor::client_state::Config, error::Error, server::Server, utils::any::Any};
use crossterm::{
    cursor::{Hide, Show},
    event::EventStream,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use derive_more::From;
use futures::StreamExt;
use http::{HeaderValue, Uri};
use std::{
    io::{StdoutLock, Write},
    path::Path,
    sync::Mutex,
};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, handshake::client::Request, Message};

#[derive(From)]
pub struct Client {
    stdout: StdoutLock<'static>,
}

impl Client {
    fn new(log_filepath: Option<&Path>) -> Result<Self, Error> {
        let stdout = std::io::stdout().lock();
        let mut client = Self { stdout };

        client.on_init(log_filepath)?;

        client.ok()
    }

    fn init_tracing(log_filepath: Option<&Path>) -> Result<(), Error> {
        let Some(log_filepath) = log_filepath else {
            return ().ok();
        };
        let writer = log_filepath.create()?.buf_writer();
        let writer = Mutex::new(writer);

        // TODO: consider using tracing-appender for writing to a file
        tracing_subscriber::fmt().with_writer(writer).json().init();

        ().ok()
    }

    fn on_init(&mut self, log_filepath: Option<&Path>) -> Result<(), Error> {
        Self::init_tracing(log_filepath)?;
        crossterm::terminal::enable_raw_mode()?;
        self.stdout
            .queue(EnterAlternateScreen)?
            .queue(Hide)?
            .queue(Clear(ClearType::All))?
            .flush()?;

        ().ok()
    }

    fn on_drop(&mut self) -> Result<(), Error> {
        crossterm::terminal::disable_raw_mode()?;
        self.stdout.queue(LeaveAlternateScreen)?.queue(Show)?.flush()?;

        ().ok()
    }

    fn config(client_args: &mut ClientArgs) -> Result<Config, Error> {
        let size = crossterm::terminal::size()?;
        let filepath = client_args.filepath.take();

        Config { size, filepath }.ok()
    }

    fn request(mut client_args: ClientArgs) -> Result<Request, Error> {
        let config = Self::config(&mut client_args)?;
        let mut request = client_args.server_address.into_client_request()?;
        let config_header = config.serialize()?;
        let config_header = HeaderValue::from_str(&config_header)?;

        request.headers_mut().insert(Server::CONFIG_HEADER_NAME, config_header);

        request.ok()
    }

    pub async fn run(client_args: ClientArgs) -> Result<(), Error> {
        let mut client = Client::new(client_args.log_filepath.as_deref())?;
        let mut events = EventStream::new();
        let request = Self::request(client_args)?;
        let (web_socket, _response) = tokio_tungstenite::connect_async(request).await?;
        let (mut sink, mut stream) = web_socket.split();

        let recv = async {
            while let Some(message_res) = stream.next().await {
                match message_res? {
                    Message::Binary(bytes) => client.stdout.write_all_and_flush(&bytes)?,
                    Message::Close(_close) => std::todo!(),
                    ignored_message => tracing::warn!(?ignored_message),
                }
            }

            ().ok()
        };
        let send = async {
            while let Some(event_res) = events.next().await {
                // TODO: figure out why this doesn't work
                // - event_res?.encode()?.into().send_to(&mut sink).await?;
                event_res?.encode()?.convert::<Message>().send_to(&mut sink).await?;
            }

            ().ok()
        };

        crate::utils::macros::select!(recv, send)
    }

    pub fn default_server_address() -> Uri {
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
