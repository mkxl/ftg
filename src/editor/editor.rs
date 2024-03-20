use crate::{
    error::Error,
    utils::{any::Any, bytes::Bytes},
};
use crossterm::event::Event as CrosstermEvent;
use derive_more::From;
use ratatui::{backend::CrosstermBackend, Terminal as RatatuiTerminal};
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

type Terminal = RatatuiTerminal<CrosstermBackend<Bytes>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub size: Option<(u16, u16)>,
}

#[derive(Debug, Deserialize, From, Serialize)]
pub enum Event {
    CrosstermEvent(CrosstermEvent),
    Config(Config),
}

pub struct View {
    buffer_id: Ulid,
    terminal: Terminal,
    bytes: Bytes,
}

pub struct ClientState {
    view_id: Ulid,
}

#[derive(Default)]
pub struct Editor {
    buffers: HashMap<Ulid, Rope>,
    views: HashMap<Ulid, View>,
    clients: HashMap<Ulid, Option<ClientState>>,
}

impl Editor {
    pub fn new_client(&mut self) -> Ulid {
        let client_id = Ulid::new();

        self.clients.insert(client_id, None);

        client_id
    }

    fn new_view(buffer_id: Ulid) -> Result<View, Error> {
        let bytes = Bytes::default();
        let backend = CrosstermBackend::new(bytes.clone());
        let terminal = Terminal::new(backend)?;
        let view = View {
            buffer_id,
            terminal,
            bytes,
        };

        view.ok()
    }

    pub fn render(&mut self, client_id: &Ulid) -> Option<Vec<u8>> {
        let client_state = self.clients.get(client_id)?.as_ref()?;
        let view = self.views.get_mut(&client_state.view_id)?;

        view.terminal.draw(|frame| std::todo!());

        view.bytes.take().some()
    }

    pub async fn feed(&self, client_id: &Ulid, event: Event) -> Result<bool, Error> {
        tracing::info!(?event);

        false.ok()
    }
}
