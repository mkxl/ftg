use crate::{
    cli::ServerArgs,
    editor::{window::Args as WindowArgs, Editor},
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
use serde_json::Error as SerdeJsonError;
use std::{net::Ipv4Addr, sync::Arc};

#[derive(Constructor)]
pub struct Server {
    editor: Arc<Mutex<Editor>>,
}

#[OpenApi]
impl Server {
    const API_PATH: &'static str = "/";
    const API_TITLE: &'static str = std::env!("CARGO_PKG_NAME");
    const API_VERSION: &'static str = std::env!("CARGO_PKG_VERSION");
    // TODO: resolve
    // pub const CONFIG_HEADER_NAME: &'static str = "x-ftg-window-args";
    pub const CONFIG_HEADER_NAME: &'static str = "window_args";
    pub const DEFAULT_HOST: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
    pub const DEFAULT_PORT: u16 = 8080;

    pub async fn serve(server_args: ServerArgs) -> Result<(), Error> {
        Self::init_tracing();

        let server = Self::new(Arc::default());
        let address = (server_args.host, server_args.port);
        let tcp_listener = TcpListener::bind(address);
        let poem_server = PoemServer::new(tcp_listener);
        let open_api_service = OpenApiService::new(server, Self::API_TITLE, Self::API_VERSION);
        let route = Route::new().nest(Self::API_PATH, open_api_service).with(Tracing);

        poem_server.run(route).await?.ok()
    }

    fn init_tracing() {
        tracing_subscriber::fmt().json().init();
    }

    async fn run(
        window_args: &WindowArgs,
        editor: Arc<Mutex<Editor>>,
        web_socket_stream: WebSocketStream,
    ) -> Result<(), Error> {
        let ready = std::future::ready(());
        let window_id = editor.lock().new_window(window_args)?;
        let (mut web_socket_sink, mut web_socket_stream) = web_socket_stream.split();

        loop {
            // NOTE: can't use else branch bc tokio::select! waits for first future to complete
            tokio::select! {
                message_res_opt = web_socket_stream.next() => {
                    let Some(message_res) = message_res_opt else { break; };
                    let end = match message_res? {
                        Message::Binary(bytes) => editor.lock().feed(&window_id, &bytes.decode()?)?,
                        Message::Close(_close) => std::todo!(),
                        ignored_message => tracing::warn!(?ignored_message).with(false),
                    };

                    if end {
                        break;
                    }
                }
                () = ready.clone() => {
                    let Some(bytes) = editor.lock().render(&window_id)? else { std::todo!(); };

                    if !bytes.is_empty() {
                        bytes.binary_message().send_to(&mut web_socket_sink).await?;
                    }
                }
            }
        }

        ().ok()
    }

    #[allow(clippy::needless_pass_by_value)]
    fn deserialization_poem_error(err: SerdeJsonError) -> PoemError {
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
        let window_args = window_args.deserialize().map_err(Self::deserialization_poem_error)?;
        let editor = self.editor.clone();
        let web_socket_upgraded = web_socket.on_upgrade(|web_socket_stream| async move {
            Self::run(&window_args, editor, web_socket_stream).await.error();
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
