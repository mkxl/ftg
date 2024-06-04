use crate::{
    config::Config,
    editor::{
        buffer::buffer::Buffer,
        command::Command,
        keymap::{Context, Keymap},
        view::view::{View, ViewState},
        window::{Window, WindowArgs},
    },
    error::Error,
    utils::{any::Any, container::Container},
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::{io::Error as IoError, path::Path};
use ulid::Ulid;

macro_rules! active_view {
    ($self:ident, $window_id:ident) => {{
        let window = $self.windows.get_mut($window_id)?;
        let (before, view, after) = window.active_view();
        let buffer = $self.buffers.get_mut(&view.buffer_id())?;
        let view_state = ViewState { buffer, before, after };

        // NOTE: need turbofish here because E return type parameter is unspecified and cargo complains
        (view, view_state).ok::<Error>()
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

macro_rules! mouse_pattern {
    ($variant:ident) => {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::$variant,
            ..
        })
    };
}

pub struct Editor {
    buffers: Container<Buffer>,
    windows: Container<Window>,
    keymap: Keymap,
}

impl Editor {
    const BUFFERS_CONTAINER_NAME: &'static str = "buffers";
    const WINDOWS_CONTAINER_NAME: &'static str = "windows";

    pub fn new(config: Config) -> Self {
        let buffers = Container::new(Self::BUFFERS_CONTAINER_NAME.into());
        let windows = Container::new(Self::WINDOWS_CONTAINER_NAME.into());
        let keymap = Keymap::new(config.keymap);

        Self {
            buffers,
            windows,
            keymap,
        }
    }

    fn views(&mut self, args: WindowArgs) -> Result<Vec<View>, Error> {
        let rect = args.size().rect();
        let filepaths = args.into_paths();
        let filepaths = if filepaths.is_empty() {
            None.once().left()
        } else {
            filepaths.into_iter().map(Some).right()
        };
        let views = filepaths.map(|filepath_opt| {
            let buffer_id = self.get_buffer_id(filepath_opt.as_deref())?;
            let view = View::new(buffer_id, rect, filepath_opt)?;

            view.ok()
        });

        views.collect()
    }

    pub fn new_window(&mut self, args: WindowArgs) -> Result<Ulid, Error> {
        let views = self.views(args)?;
        let window = Window::new(views);
        let window_id = window.id();

        self.windows.insert(window);

        window_id.ok()
    }

    fn get_buffer_id(&mut self, filepath: Option<&Path>) -> Result<Ulid, IoError> {
        let Some(filepath) = filepath else {
            return self.buffers.insert(Buffer::default()).id().ok();
        };

        if let Ok(buffer) = self.buffers.get(&filepath.inode_id()?) {
            buffer.id().ok()
        } else {
            self.buffers.insert(Buffer::from_filepath(filepath)?).id().ok()
        }
    }

    fn active_view(&mut self, window_id: &Ulid) -> Result<(&mut View, ViewState), Error> {
        active_view!(self, window_id)
    }

    pub fn render(&mut self, window_id: &Ulid) -> Result<Option<Vec<u8>>, Error> {
        let (view, view_state) = self.active_view(window_id)?;

        view.render(&view_state)?.some().ok()
    }

    pub fn feed(&mut self, window_id: &Ulid, event: Event) -> Result<bool, Error> {
        // NOTE: use active_view!() macro rather than self.active_view() method to prevent
        // `cannot borrow `self.keymap` as immutable because it is also borrowed as mutable`
        let (view, ViewState { buffer, .. }) = active_view!(self, window_id)?;

        match self.keymap.get(view.context(), &[event]) {
            (_, Ok(Command::Quit)) => return true.ok(),
            (_, Err(&[Event::Resize(width, height)])) => view.resize(width, height)?,
            (_, Err(&[mouse_pattern!(ScrollUp)])) => view.move_up(1),
            (_, Err(&[mouse_pattern!(ScrollDown)])) => view.move_down(buffer, 1),
            (Context::Buffer, Ok(Command::MoveUp { count })) => view.move_up(*count),
            (Context::Buffer, Ok(Command::MoveDown { count })) => view.move_down(buffer, *count),
            (Context::Buffer, Ok(Command::MoveLeft)) => view.move_left(),
            (Context::Buffer, Ok(Command::MoveRight)) => view.move_right(),
            (Context::Buffer, Ok(Command::Save)) => view.save(buffer).warn().unit(),
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
