use crate::{
    config::Config,
    editor::{
        buffer::buffer::Buffer,
        keymap::{Command, Context, Keymap},
        view::view::View,
        window::{Window, WindowArgs},
    },
    error::Error,
    utils::{any::Any, container::Container},
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::{io::Error as IoError, path::Path};
use ulid::Ulid;

macro_rules! get_mut {
    ($self:ident, $window_id:ident) => {{
        let window = $self.windows.get_mut($window_id);
        let view = if let Some(ref window) = window {
            $self.views.get_mut(&window.primary_view_id())
        } else {
            None
        };
        let buffer = if let Some(ref view) = view {
            $self.buffers.get_mut(&view.buffer_id())
        } else {
            None
        };

        GetMut { window, view, buffer }
    }};
}

macro_rules! key_pattern {
    ($chr:ident) => {
        Event::Key(KeyEvent {
            code: KeyCode::Char($chr),
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            ..
        })
    };
}

pub struct GetMut<'a> {
    window: Option<&'a mut Window>,
    view: Option<&'a mut View>,
    buffer: Option<&'a mut Buffer>,
}

pub struct Editor {
    buffers: Container<Buffer>,
    views: Container<View>,
    windows: Container<Window>,
    keymap: Keymap,
}

impl Editor {
    pub fn new(config: Config) -> Self {
        let keymap = Keymap::new(config.keymap);

        Self {
            buffers: Container::default(),
            views: Container::default(),
            windows: Container::default(),
            keymap,
        }
    }

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
        get_mut!(self, window_id)
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

    pub fn feed(&mut self, window_id: &Ulid, event: Event) -> Result<bool, Error> {
        // TODO: remove; currently log for debugging purposes for when the client hangs
        tracing::info!(feed_event = ?event);

        // NOTE: use get_mut!() macro rather than self.get_mut() method to prevent
        // `cannot borrow `self.keymap` as immutable because it is also borrowed as mutable`
        let GetMut {
            view: Some(view),
            buffer: Some(buffer),
            ..
        } = get_mut!(self, window_id)
        else {
            return true.ok();
        };

        match self.keymap.get(view.context(), &[event]) {
            (_, Ok(Command::Quit)) => return true.ok(),
            (_, Err(&[Event::Resize(width, height)])) => view.resize(width, height)?,
            (Context::Buffer, Ok(Command::MoveUp)) => view.move_up(),
            (Context::Buffer, Ok(Command::MoveDown)) => view.move_down(buffer),
            (Context::Buffer, Ok(Command::MoveLeft)) => view.move_left(),
            (Context::Buffer, Ok(Command::MoveRight)) => view.move_right(),
            (Context::Buffer, Ok(Command::Search)) => view.begin_search(),
            (Context::Buffer, Err(&[key_pattern!(chr)])) => view.insert_char(buffer, chr),
            (Context::Search, Ok(Command::Submit)) => view.submit_search(buffer),
            (Context::Search, Ok(Command::Close)) => view.close_search(),
            (Context::Search, Err(&[key_pattern!(chr)])) => view.push_search(chr),
            (context, ignored_result) => tracing::info!(view.context = ?context, ?ignored_result),
        }

        false.ok()
    }
}
