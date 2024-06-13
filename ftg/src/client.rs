use crate::{cli_args::CliArgs, editor::window::window::WindowArgs, error::Error, server::Server, utils::any::Any};
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture, EventStream},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use derive_more::From;
use futures::StreamExt;
use http::HeaderValue;
use reqwest::Client as ReqwestClient;
use std::io::{StdoutLock, Write};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, handshake::client::Request, Message};

#[derive(From)]
pub struct Client {
    stdout: StdoutLock<'static>,
}

impl Client {
    fn new() -> Result<Self, Error> {
        let stdout = std::io::stdout().lock();
        let mut client = Self { stdout };

        client.on_init()?;

        client.ok()
    }

    fn on_init(&mut self) -> Result<(), Error> {
        crossterm::terminal::enable_raw_mode()?;
        self.stdout
            .queue(EnableMouseCapture)?
            .queue(EnterAlternateScreen)?
            .queue(Hide)?
            .queue(Clear(ClearType::All))?
            .flush()?;

        ().ok()
    }

    fn on_drop(&mut self) -> Result<(), Error> {
        crossterm::terminal::disable_raw_mode()?;
        self.stdout
            .queue(DisableMouseCapture)?
            .queue(LeaveAlternateScreen)?
            .queue(Show)?
            .flush()?;

        ().ok()
    }

    fn window_args(cli_args: CliArgs) -> Result<WindowArgs, Error> {
        let terminal_shape = crossterm::terminal::size()?;
        let current_dirpath = std::env::current_dir()?;
        let window_args = WindowArgs::new(terminal_shape, &current_dirpath, cli_args.paths);

        window_args.ok()
    }

    fn request(cli_args: CliArgs) -> Result<Request, Error> {
        let mut request = Self::url("ws", &cli_args).into_client_request()?;
        let window_args_header = Self::window_args(cli_args)?.serialize()?;
        let window_args_header = HeaderValue::from_str(&window_args_header)?;

        request
            .headers_mut()
            .insert(Server::WINDOW_ARGS_HEADER_NAME, window_args_header);

        request.ok()
    }

    async fn run_client(cli_args: CliArgs) -> Result<(), Error> {
        let mut client = Client::new()?;
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
            // TODO: figure out clone here
            let server_cli_args = cli_args.clone();
            let server_future = Server::serve(&server_cli_args);
            let client_future = Self::run_client(cli_args);

            // NOTE-c9481a: server_future and client_future do not implement Unpin necessitating futures::pin_mut!()
            futures::pin_mut!(server_future, client_future);

            server_future.select(client_future).await
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.on_drop().error();
    }
}
