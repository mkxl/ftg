use crate::{
    cli::ServerArgs,
    editor::editor::Editor,
    error::Error,
    utils::{any::Any, lock::Lock, web_socket_upgraded::WebSocketUpgraded},
};
use derive_more::Constructor;
use futures::StreamExt;
use poem::{
    listener::TcpListener,
    middleware::Tracing,
    web::websocket::{Message, WebSocket, WebSocketStream},
    EndpointExt, Route, Server as PoemServer,
};
use poem_openapi::{OpenApi, OpenApiService};
use std::{net::Ipv4Addr, time::Duration};

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

    async fn run(editor_recv: Lock<Editor>, web_socket_stream: WebSocketStream) -> Result<(), Error> {
        let client_id = editor_recv.get().await.new_client();
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
            while let Some(state) = editor_send.get().await.state(&client_id) {
                state.encode()?.binary_message().send_to(&mut sink).await?;
                interval.tick().await;
            }

            ().ok()
        };

        crate::utils::macros::select!(recv, send)
    }

    // NOTE: OpenApi macro requires that endpoint methods be async
    #[allow(clippy::unused_async)]
    #[oai(method = "get", path = "/")]
    async fn root(&self, web_socket: WebSocket) -> WebSocketUpgraded {
        let editor = self.editor.clone();
        let web_socket_upgraded = web_socket.on_upgrade(|web_socket_stream| async move {
            Self::run(editor, web_socket_stream).await.error();
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
        web_socket_upgraded.boxed().into()
    }
}
