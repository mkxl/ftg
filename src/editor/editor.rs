use crate::{
    error::Error,
    utils::{any::Any, bytes::Bytes},
};
use crossterm::event::Event;
use derive_more::Constructor;
use itertools::Itertools;
use ratatui::{backend::CrosstermBackend, widgets::Paragraph, Terminal as RatatuiTerminal};
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use ulid::Ulid;

type Terminal = RatatuiTerminal<CrosstermBackend<Bytes>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub size: (u16, u16),
    pub filepath: Option<PathBuf>,
}

pub struct View {
    buffer_id: Ulid,
    terminal: Terminal,
    bytes: Bytes,
}

#[derive(Constructor)]
pub struct ClientState {
    view_id: Ulid,
    config: Config,
}

#[derive(Default)]
pub struct Editor {
    buffers: HashMap<Ulid, Rope>,
    views: HashMap<Ulid, View>,
    clients: HashMap<Ulid, ClientState>,
}

impl Editor {
    pub fn new_client(&mut self, config: Config) -> Result<Ulid, Error> {
        let client_id = Ulid::new();
        let view_id = Ulid::new();
        let buffer_id = Ulid::new();
        let client_state = ClientState::new(view_id, config);
        let rope = if let Some(filepath) = &client_state.config.filepath {
            filepath.rope()?
        } else {
            Rope::new()
        };
        let view = Self::new_view(buffer_id)?;

        self.buffers.insert(buffer_id, rope);
        self.views.insert(view_id, view);
        self.clients.insert(client_id, client_state);

        client_id.ok()
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

    pub fn render(&mut self, client_id: &Ulid) -> Result<Option<Vec<u8>>, Error> {
        // TODO: what to do if client_id or view_id isn't found
        let Some(client_state) = self.clients.get(client_id) else {
            return None.ok();
        };
        let Some(view) = self.views.get_mut(&client_state.view_id) else {
            return None.ok();
        };
        let Some(rope) = self.buffers.get(&view.buffer_id) else {
            return None.ok();
        };
        let text = rope.lines().take(client_state.config.size.1 as usize).join("");
        let paragraph = Paragraph::new(text);

        view.terminal.draw(|frame| {
            frame.render_widget(paragraph, frame.size());
        })?;

        view.bytes.take().some().ok()
    }

    pub async fn feed(&self, client_id: &Ulid, event: Event) -> Result<bool, Error> {
        tracing::info!(?event);

        false.ok()
    }
}
