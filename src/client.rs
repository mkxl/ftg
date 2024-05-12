use crate::{cli_args::CliArgs, editor::window::WindowArgs, error::Error, server::Server, utils::any::Any};
use crossterm::{
    cursor::{Hide, Show},
    event::EventStream,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use derive_more::From;
use futures::StreamExt;
use http::HeaderValue;
use reqwest::Client as ReqwestClient;
use std::{
    io::{StdoutLock, Write},
    path::Path,
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
        let log_file = log_filepath.create()?;

        // TODO: consider using tracing-appender for writing to a file
        tracing_subscriber::fmt().with_writer(log_file).json().init();

        ().ok()
    }

    fn on_init(&mut self, log_filepath: Option<&Path>) -> Result<(), Error> {
        // Self::init_tracing(log_filepath)?;
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

    fn window_args(cli_args: &mut CliArgs) -> Result<WindowArgs, Error> {
        let size = crossterm::terminal::size()?;
        let filepath = cli_args.filepath.take();
        let window_args = WindowArgs { size, filepath };

        window_args.ok()
    }

    fn request(mut cli_args: CliArgs) -> Result<Request, Error> {
        let window_args = Self::window_args(&mut cli_args)?;
        let mut request = Self::url("ws", &cli_args).into_client_request()?;
        let window_args_header = window_args.serialize()?;
        let window_args_header = HeaderValue::from_str(&window_args_header)?;

        request
            .headers_mut()
            .insert(Server::WINDOW_ARGS_HEADER_NAME, window_args_header);

        request.ok()
    }

    async fn run_client(cli_args: CliArgs) -> Result<(), Error> {
        let mut client = Client::new(cli_args.log_filepath.as_deref())?;
        let mut events = EventStream::new();
        let request = Self::request(cli_args)?;
        let (mut web_socket, _response) = tokio_tungstenite::connect_async(request).await?;

        loop {
            tokio::select! {
                message_res_opt = web_socket.next() => {
                    match message_res_opt {
                        Some(Ok(Message::Binary(bytes))) => client.stdout.write_all_and_flush(&bytes)?,
                        Some(Ok(Message::Close(_close))) => std::todo!(),
                        Some(Ok(ignored_message)) => tracing::warn!(?ignored_message),
                        Some(message_res) => message_res?.unit(),
                        None => std::todo!(),
                    }
                }
                event_res_opt = events.next() => {
                    let Some(event_res) = event_res_opt else { std::todo!(); };

                    // TODO: figure out why this doesn't work
                    // - [event_res?.encode()?.into().send_to(&mut sink).await?;]
                    event_res?.encode()?.convert::<Message>().send_to(&mut web_socket).await?;
                }
            }
        }
    }

    fn url(protocol: &str, cli_args: &CliArgs) -> String {
        std::format!("{protocol}://{host}:{port}", host = cli_args.host, port = cli_args.port)
    }

    async fn server_is_running(cli_args: &CliArgs) -> Result<bool, Error> {
        let url = Self::url("http", cli_args);
        let response_res = ReqwestClient::new().head(url).send().await;
        let Err(reqwest_err) = response_res else {
            return true.ok();
        };

        if reqwest_err.is_request() {
            return false.ok();
        }

        Err(reqwest_err)?
    }

    pub async fn run(cli_args: CliArgs) -> Result<(), Error> {
        if Self::server_is_running(&cli_args).await? {
            Self::run_client(cli_args).await
        } else {
            let server_future = Server::serve(&cli_args).await?;
            let client_future = Self::run_client(cli_args);

            tokio::select! {
                res = server_future => res??.ok(),
                res = client_future => res,
            }
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.on_drop().error();
    }
}
