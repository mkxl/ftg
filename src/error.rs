use derive_more::{Display, From};
use postcard::Error as PostcardError;
use std::io::Error as IoError;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

// NOTE:
// - Error must implement Debug to be used as E in fn main() -> Result<(), E>
// - Error must implement Display for Any::error()
#[derive(Debug, Display, From)]
pub enum Error {
    Io(IoError),
    Postcard(PostcardError),
    Tungstenite(TungsteniteError),
}
