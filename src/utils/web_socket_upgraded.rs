use derive_more::From;
use poem::{web::websocket::BoxWebSocketUpgraded, IntoResponse, Response};
use poem_openapi::{
    registry::{MetaResponse, MetaResponses, Registry},
    ApiResponse,
};

#[derive(From)]
pub struct WebSocketUpgraded(BoxWebSocketUpgraded);

impl IntoResponse for WebSocketUpgraded {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

// NOTE:
// - copied from [https://docs.rs/poem-openapi/latest/src/poem_openapi/base.rs.html#343-360]
// - normally, would either (1) call trait methods on wrapped type, but because the wrapped type BoxWebSocketUpgraded
//   doesn't implement the trait that won't work or (2) call trait methods on WebSocketUpgraded<F>, but i can't easily
//   construct a reference to a type F that statisfies the necessary trait bounds, that also won't work
// - so we settle for copying
impl ApiResponse for WebSocketUpgraded {
    fn meta() -> MetaResponses {
        MetaResponses {
            responses: vec![MetaResponse {
                description: "A websocket response",
                status: Some(101),
                content: vec![],
                headers: vec![],
            }],
        }
    }

    fn register(_registry: &mut Registry) {}
}
