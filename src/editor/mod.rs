pub mod buffer;
pub mod terminal;
pub mod view;
pub mod window;

use crate::{
    editor::{
        buffer::Buffer,
        view::View,
        window::{Args as WindowArgs, Window},
    },
    error::Error,
    utils::{
        any::Any,
        container::{Container, Identifiable},
    },
};
use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use std::{io::Error as IoError, path::Path};
use ulid::Ulid;

pub struct GetMut<'a> {
    window: Option<&'a mut Window>,
    view: Option<&'a mut View>,
    buffer: Option<&'a mut Buffer>,
}

#[derive(Default)]
pub struct Editor {
    buffers: Container<Buffer>,
    views: Container<View>,
    windows: Container<Window>,
}

impl Editor {
    pub fn new_window(&mut self, args: WindowArgs) -> Result<Ulid, Error> {
        let buffer_id = self.buffer(args.filepath.as_deref())?;
        let view = View::new(buffer_id, args)?;
        let window = Window::new(&view);
        let window_id = window.id();

        self.views.insert(view);
        self.windows.insert(window);

        window_id.ok()
    }

    fn buffer(&mut self, filepath: Option<&Path>) -> Result<Ulid, IoError> {
        let Some(filepath) = filepath else {
            return self.buffers.insert(Buffer::default()).id().ok();
        };

        if let Some(buffer) = self.buffers.get_mut(&filepath.inode_id()?) {
            buffer.id().ok()
        } else {
            self.buffers.insert(Buffer::from_filepath(filepath)?).id().ok()
        }
    }

    fn get_mut(&mut self, window_id: &Ulid) -> GetMut {
        let window = self.windows.get_mut(window_id);
        let view = if let Some(ref window) = window {
            self.views.get_mut(window.primary_view_id())
        } else {
            None
        };
        let buffer = if let Some(ref view) = view {
            self.buffers.get_mut(&view.buffer_id())
        } else {
            None
        };

        GetMut { window, view, buffer }
    }

    pub fn render(&mut self, window_id: &Ulid) -> Result<Option<Vec<u8>>, Error> {
        let GetMut {
            window: Some(window),
            view: Some(view),
            buffer: Some(buffer),
        } = self.get_mut(window_id)
        else {
            return None.ok();
        };

        view.render(window, buffer)?.some().ok()
    }

    pub fn feed(&mut self, window_id: &Ulid, event: &Event) -> Result<bool, Error> {
        // TODO: remove; currently log for debugging purposes for when the client hangs
        tracing::info!(feed_event = ?event);

        let GetMut {
            view: Some(view),
            buffer: Some(buffer),
            ..
        } = self.get_mut(window_id)
        else {
            return true.ok();
        };

        match event {
            Event::Key(KeyEvent { code: KeyCode::Up, .. })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                ..
            }) => view.move_up(),
            Event::Key(KeyEvent {
                code: KeyCode::Down, ..
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                ..
            }) => view.move_down(buffer),
            Event::Key(KeyEvent {
                code: KeyCode::Left, ..
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollLeft,
                ..
            }) => view.move_left(),
            Event::Key(KeyEvent {
                code: KeyCode::Right, ..
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollRight,
                ..
            }) => view.move_right(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => return true.ok(),
            Event::Resize(width, height) => view.resize(*width, *height)?,
            _ => {}
        }

        false.ok()
    }
}
