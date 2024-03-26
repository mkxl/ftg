use crate::{
    cli::ServerArgs,
    editor::{client_state::Config, editor::Editor},
    error::Error,
    utils::{any::Any, lock::Lock, web_socket_upgraded::WebSocketUpgraded},
};
use derive_more::Constructor;
use futures::{future::Either, StreamExt};
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
use std::{net::Ipv4Addr, sync::Arc, time::Duration};
use tokio_stream::wrappers::IntervalStream;

#[derive(Constructor)]
pub struct Server {
    editor: Lock<Editor>,
}

#[OpenApi]
impl Server {
    const API_PATH: &'static str = "/";
    const API_TITLE: &'static str = std::env!("CARGO_PKG_NAME");
    const API_VERSION: &'static str = std::env!("CARGO_PKG_VERSION");
    const INTERVAL_DURATION: Duration = Duration::from_millis(50);
    // TODO: resolve
    // pub const CONFIG_HEADER_NAME: &'static str = "x-ftg-config";
    pub const CONFIG_HEADER_NAME: &'static str = "config";
    pub const DEFAULT_HOST: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
    pub const DEFAULT_PORT: u16 = 8080;

    pub async fn serve(server_args: ServerArgs) -> Result<(), Error> {
        Self::init_tracing();

        let editor = Editor::default().locked();
        let server = Self::new(editor);
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

    // async fn run2(config: Config, editor: Arc<Mutex<Editor>>, web_socket_stream: WebSocketStream) -> Result<(), Error> {
    //     let client_id = editor.lock().new_client(config)?;
    //     let interval = tokio::time::interval(Self::INTERVAL_DURATION);
    //     let interval_stream = IntervalStream::new(interval);
    //     let left = Either::Left(interval_stream);
    //     let right = Either::Right(web_socket_stream);
    //     let joint: () = futures::stream::select(left, right);

    //     while let Some(either) = joint.next().await {
    //         match either {
    //             Either::Left(_instant) => {
    //                 let Some(bytes) = editor.lock().render(&client_id)? else {
    //                     break;
    //                 };

    //                 if !bytes.is_empty() {
    //                     bytes.binary_message().send_to(&mut web_socket_stream).await?;
    //                 }
    //             }
    //             Either::Right(message_res) => {
    //                 let end = match message_res? {
    //                     Message::Binary(bytes) => editor.lock().feed(&client_id, bytes.decode()?).await?,
    //                     Message::Close(_close) => std::todo!(),
    //                     ignored_message => tracing::warn!(?ignored_message).with(false),
    //                 };

    //                 if end {
    //                     break;
    //                 }
    //             }
    //         }
    //     }

    //     ().ok()
    // }

    // TODO: consider instead of having recv/send loops, just doing an either situation with events/web_socket_stream
    // so that i can .close().await properly from the recv loop; might mean i can get rid of the lock on editor (
    // actually not sure if the tokio mutex is still needed - don't know if i still need to await across locks)
    async fn run(config: Config, editor_recv: Lock<Editor>, web_socket_stream: WebSocketStream) -> Result<(), Error> {
        let client_id = editor_recv.get().await.new_client(config)?;
        let (mut sink, mut stream) = web_socket_stream.split();
        let mut interval = tokio::time::interval(Self::INTERVAL_DURATION);
        let editor_send = editor_recv.clone();
        let recv = async {
            while let Some(message_res) = stream.next().await {
                let end = match message_res? {
                    Message::Binary(bytes) => editor_recv.get().await.feed(&client_id, bytes.decode()?).await?,
                    Message::Close(_close) => std::todo!(),
                    ignored_message => tracing::warn!(?ignored_message).with(false),
                };

                if end {
                    break;
                }
            }

            ().ok()
        };
        let send = async {
            while let Some(bytes) = editor_send.get().await.render(&client_id)? {
                if !bytes.is_empty() {
                    bytes.binary_message().send_to(&mut sink).await?;
                }

                interval.tick().await;
            }

            ().ok()
        };

        crate::utils::macros::select!(recv, send)
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
        Header(config): Header<String>,
    ) -> Result<WebSocketUpgraded, PoemError> {
        let config = config.deserialize().map_err(Self::deserialization_poem_error)?;
        let editor = self.editor.clone();
        let web_socket_upgraded = web_socket.on_upgrade(|web_socket_stream| async move {
            Self::run(config, editor, web_socket_stream).await.error();
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
