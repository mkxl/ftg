use crate::{
    cli::ServerArgs,
    config::Config,
    editor::{editor::Editor, window::WindowArgs},
    error::Error,
    utils::{any::Any, web_socket_upgraded::WebSocketUpgraded},
};
use derive_more::Constructor;
use futures::StreamExt;
use parking_lot::Mutex;
use poem::{
    http::StatusCode,
    listener::TcpListener,
    middleware::Tracing,
    web::websocket::{Message, WebSocket, WebSocketStream},
    EndpointExt, Error as PoemError, Route, Server as PoemServer,
};
use poem_openapi::{param::Header, OpenApi, OpenApiService};
use std::{fmt::Display, net::Ipv4Addr, sync::Arc};

#[derive(Constructor)]
pub struct Server {
    editor: Arc<Mutex<Editor>>,
}

#[OpenApi]
impl Server {
    const API_PATH: &'static str = "/";
    const API_TITLE: &'static str = std::env!("CARGO_PKG_NAME");
    const API_VERSION: &'static str = std::env!("CARGO_PKG_VERSION");
    const DEFAULT_CONFIG_STR: &'static str = std::include_str!("config.yaml");
    // TODO: resolve
    // pub const WINDOW_ARGS_HEADER_NAME: &'static str = "x-ftg-window-args";
    pub const WINDOW_ARGS_HEADER_NAME: &'static str = "window_args";

    pub async fn serve(server_args: ServerArgs) -> Result<(), Error> {
        Self::init_tracing();

        let config = Self::config(&server_args)?;
        let address = (config.host, config.port);
        let tcp_listener = TcpListener::bind(address);
        let poem_server = PoemServer::new(tcp_listener);
        let editor = Editor::new(config).mutex().arc();
        let server = Self::new(editor);
        let open_api_service = OpenApiService::new(server, Self::API_TITLE, Self::API_VERSION);
        let route = Route::new().nest(Self::API_PATH, open_api_service).with(Tracing);

        poem_server.run(route).await?.ok()
    }

    fn config(server_args: &ServerArgs) -> Result<Config, Error> {
        // TODO: why is the turbofish necessary
        if let Some(config_filepath) = &server_args.config_filepath {
            config_filepath
                .read_to_string()?
                .deserialize_from_yaml::<Config>()?
                .ok()
        } else {
            Self::DEFAULT_CONFIG_STR.deserialize_from_yaml::<Config>()?.ok()
        }
    }

    pub fn default_host() -> Ipv4Addr {
        Ipv4Addr::UNSPECIFIED
    }

    pub fn default_port() -> u16 {
        8080
    }

    fn init_tracing() {
        tracing_subscriber::fmt().json().init();
    }

    async fn run(
        window_args: WindowArgs,
        editor: Arc<Mutex<Editor>>,
        mut web_socket_stream: WebSocketStream,
    ) -> Result<(), Error> {
        let window_id = editor.lock().new_window(window_args)?;

        loop {
            // NOTE: i want to always operate on the next websocket message, if present, and if not, send the bytes to
            // the client; i can't use the the tokio::select! else branch bc it waits for the first future to complete
            // and checks if the branch is valid before falling back to the else branch; bc of this we use
            // std::future::ready(()) instead and yield if no bytes are to be written [a47d22]
            tokio::select! {
                biased;
                message_res_opt = web_socket_stream.next() => {
                    let Some(message_res) = message_res_opt else { break; };
                    let end = match message_res? {
                        Message::Binary(bytes) => editor.lock().feed(&window_id, bytes.decode()?)?,
                        Message::Close(_close) => std::todo!(),
                        ignored_message => tracing::warn!(?ignored_message).with(false),
                    };

                    if end {
                        break;
                    }
                }
                () = std::future::ready(()) => {
                    let Some(bytes) = editor.lock().render(&window_id)? else { std::todo!(); };

                    if bytes.is_empty() {
                        tokio::task::yield_now().await;
                    } else {
                        bytes.binary_message().send_to(&mut web_socket_stream).await?;
                    }
                }
            }
        }

        tracing::info!(ending_session_for_window_id = %window_id);

        ().ok()
    }

    fn bad_request(err: impl Display) -> PoemError {
        PoemError::from_string(err.to_string(), StatusCode::BAD_REQUEST)
    }

    // NOTE: OpenApi macro requires that endpoint methods be async
    #[allow(clippy::unused_async)]
    #[oai(method = "get", path = "/")]
    async fn root(
        &self,
        web_socket: WebSocket,
        Header(window_args): Header<String>,
    ) -> Result<WebSocketUpgraded, PoemError> {
        let window_args = window_args.deserialize_from_json().map_err(Self::bad_request)?;
        let editor = self.editor.clone();
        let web_socket_upgraded = web_socket.on_upgrade(|web_socket_stream| async {
            Self::run(window_args, editor, web_socket_stream).await.error();
        });

        // NOTE:
        // - web_socket_upgraded is of type poem::web::websocket::WebSocketUpgraded<{unnameable-closure}> and so the
        //   method's return type can't be written out concretely
        // - normally, would just use `impl ApiResponse + IntoResponse` as the return type (as these are the traits the
        //   OpenApi macro requires it to implement), but the OpenApi macro also requires the return type be concrete
        //   (see `cargo expand` for why)
        // - the seeming solution would be to call .boxed() to get a BoxWebSocketUpgraded which does implement
        //   IntoResponse; however it doesn't implement ApiResponse (likely an oversight, so probably happening soon);
        //   so we create a wrapper and have it implement both
        web_socket_upgraded.boxed().convert::<WebSocketUpgraded>().ok()
    }
}
