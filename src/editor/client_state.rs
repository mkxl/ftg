use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ulid::Ulid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub size: (u16, u16),
    pub filepath: Option<PathBuf>,
}

#[derive(Constructor)]
pub struct ClientState {
    view_id: Ulid,
    config: Config,
}

impl ClientState {
    pub fn view_id(&self) -> &Ulid {
        &self.view_id
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
