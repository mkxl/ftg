use derive_more::{Display, From};
use http::{header::InvalidHeaderValue, Error as HttpError};
use postcard::Error as PostcardError;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;
use serde_yaml::Error as SerdeYamlError;
use std::io::Error as IoError;
use tokio::task::JoinError as TokioJoinError;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;
use ulid::Ulid;

// NOTE:
// - Error must implement Debug to be used as E in fn main() -> Result<(), E>
// - Error must implement Display for Any::error()
#[derive(Debug, Display, From)]
pub enum Error {
    Http(HttpError),
    InvalidHeaderValue(InvalidHeaderValue),
    Io(IoError),
    Postcard(PostcardError),
    Reqwest(ReqwestError),
    SerdeJson(SerdeJsonError),
    SerdeYaml(SerdeYamlError),
    TokioJoin(TokioJoinError),
    Tungstenite(TungsteniteError),

    #[display(fmt = "unknown {_0} ID {_1}")]
    UnknownItem(String, Ulid),
}
