use derive_more::{Display, From};
use http::{header::InvalidHeaderValue, Error as HttpError};
use postcard::Error as PostcardError;
use serde_json::Error as SerdeJsonError;
use serde_yaml::Error as SerdeYamlError;
use std::io::Error as IoError;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

// NOTE:
// - Error must implement Debug to be used as E in fn main() -> Result<(), E>
// - Error must implement Display for Any::error()
#[derive(Debug, Display, From)]
pub enum Error {
    Http(HttpError),
    InvalidHeaderValue(InvalidHeaderValue),
    Io(IoError),
    Postcard(PostcardError),
    SerdeJson(SerdeJsonError),
    SerdeYaml(SerdeYamlError),
    Tungstenite(TungsteniteError),
}
