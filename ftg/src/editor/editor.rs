use crate::{
    config::Config,
    editor::{
        buffer::buffer::Buffer,
        color_scheme::ColorScheme,
        command::Command,
        keymap::{Context, Keymap},
        view::view::View,
        window::{Window, WindowArgs},
    },
    error::Error,
    utils::{any::Any, container::Container},
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::{io::Error as IoError, path::Path};
use ulid::Ulid;

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
    color_scheme: ColorScheme,
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
            color_scheme: config.color_scheme,
            buffers,
            windows,
            keymap,
        }
    }

    fn create_views(&mut self, args: WindowArgs) -> Result<Vec<View>, Error> {
        let filepaths = args.into_paths();
        let filepaths = if filepaths.is_empty() {
            None.once().left()
        } else {
            filepaths.into_iter().map(Some).right()
        };
        let views = filepaths.map(|filepath_opt| {
            let buffer_id = self.get_buffer_id(filepath_opt.as_deref())?;
            let view = View::new(buffer_id, filepath_opt)?;

            view.ok()
        });

        views.collect()
    }

    pub fn new_window(&mut self, args: WindowArgs) -> Result<Ulid, Error> {
        let terminal_area = args.terminal_shape().rect();
        let views = self.create_views(args)?;
        let window = Window::new(terminal_area, views);
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

    pub fn render(&mut self, window_id: &Ulid) -> Result<Vec<u8>, Error> {
        self.windows
            .get_mut(window_id)?
            .render(&self.buffers, &self.color_scheme)
    }

    pub fn feed(&mut self, window_id: &Ulid, event: Event) -> Result<bool, Error> {
        // TODO: remove
        tracing::info!(?event);

        let window = self.windows.get_mut(window_id)?;
        let view = window.active_view();
        let buffer = self.buffers.get_mut(&view.buffer_id())?;

        match self.keymap.get(view.context(), &[event]) {
            (_, Ok(Command::Quit)) => return true.ok(),
            (_, Err(&[Event::Resize(width, height)])) => window.resize(width, height)?,
            (_, Err(&[mouse_pattern!(ScrollUp)])) => view.scroll_up(1),
            (_, Err(&[mouse_pattern!(ScrollDown)])) => view.scroll_down(buffer, 1),
            (_, Err(&[mouse_pattern!(ScrollLeft)])) => view.scroll_left(1),
            (_, Err(&[mouse_pattern!(ScrollRight)])) => view.scroll_right(1),
            (Context::Buffer, Ok(Command::MoveBackward)) => view.move_backward(),
            (Context::Buffer, Ok(Command::MoveDown)) => view.move_down(buffer),
            (Context::Buffer, Ok(Command::MoveForward)) => view.move_forward(),
            (Context::Buffer, Ok(Command::MoveUp)) => view.move_up(buffer),
            (Context::Buffer, Ok(Command::NextView)) => window.next_view(),
            (Context::Buffer, Ok(Command::PreviousView)) => window.previous_view(),
            (Context::Buffer, Ok(Command::Save)) => view.save(buffer).warn().unit(),
            (Context::Buffer, Ok(Command::ScrollUp { count })) => view.scroll_up(*count),
            (Context::Buffer, Ok(Command::ScrollDown { count })) => view.scroll_down(buffer, *count),
            (Context::Buffer, Ok(Command::ScrollLeft { count })) => view.scroll_left(*count),
            (Context::Buffer, Ok(Command::ScrollRight { count })) => view.scroll_right(*count),
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
