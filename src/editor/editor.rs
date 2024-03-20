use crate::{error::Error, utils::any::Any};
use crossterm::event::Event as CrosstermEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub size: Option<(u16, u16)>,
}

#[derive(Debug, Deserialize, From, Serialize)]
pub enum Event {
    CrosstermEvent(CrosstermEvent),
    Config(Config),
}

#[derive(Deserialize, Serialize)]
pub struct State;

impl State {
    fn new() -> Self {
        Self
    }
}

#[derive(Default)]
pub struct Editor {
    clients: HashMap<Ulid, State>,
}

impl Editor {
    pub fn new_client(&mut self) -> Ulid {
        let client_id = Ulid::new();

        self.clients.insert(client_id, State::new());

        client_id
    }

    pub fn state(&self, client_id: &Ulid) -> Option<&State> {
        self.clients.get(client_id)
    }

    pub async fn feed(&self, client_id: &Ulid, event: Event) -> Result<bool, Error> {
        tracing::info!(?event);

        false.ok()
    }
}
