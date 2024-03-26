use crate::{
    editor::{
        buffer::Buffer,
        client_state::{ClientState, Config},
        view::View,
    },
    error::Error,
    utils::any::Any,
};
use crossterm::event::{Event, KeyCode, KeyEvent};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Default)]
pub struct Editor {
    buffers: HashMap<Ulid, Buffer>,
    views: HashMap<Ulid, View>,
    clients: HashMap<Ulid, ClientState>,
}

impl Editor {
    pub fn new_client(&mut self, config: Config) -> Result<Ulid, Error> {
        let client_id = Ulid::new();
        let view_id = Ulid::new();
        let buffer_id = Ulid::new();
        let client_state = ClientState::new(view_id, config);
        let buffer = if let Some(filepath) = &client_state.config().filepath {
            Buffer::from_filepath(filepath)?
        } else {
            Buffer::default()
        };
        let view = View::new(buffer_id, client_state.config().size.rect())?;

        self.buffers.insert(buffer_id, buffer);
        self.views.insert(view_id, view);
        self.clients.insert(client_id, client_state);

        client_id.ok()
    }

    fn get_mut(&mut self, client_id: &Ulid) -> Option<(&mut ClientState, &mut View, &mut Buffer)> {
        let client_state = self.clients.get_mut(client_id)?;
        let view = self.views.get_mut(client_state.view_id())?;
        let buffer = self.buffers.get_mut(&view.buffer_id())?;

        (client_state, view, buffer).some()
    }

    pub fn render(&mut self, client_id: &Ulid) -> Result<Option<Vec<u8>>, Error> {
        // TODO: what to do if client_id or view_id isn't found
        let Some((client_state, view, buffer)) = self.get_mut(client_id) else {
            return None.ok();
        };

        view.render(client_state, buffer)?.some().ok()
    }

    pub async fn feed(&mut self, client_id: &Ulid, event: Event) -> Result<bool, Error> {
        let Some((_client_state, view, buffer)) = self.get_mut(client_id) else {
            return true.ok();
        };

        match event {
            Event::Key(KeyEvent { code: KeyCode::Up, .. }) => view.move_up(),
            Event::Key(KeyEvent {
                code: KeyCode::Down, ..
            }) => view.move_down(buffer),
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => return true.ok(),
            Event::Resize(width, height) => view.resize(width, height)?,
            _ => {}
        }

        false.ok()
    }
}
