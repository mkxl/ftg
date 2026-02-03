use crate::{buffer_map::BufferMap, command::Command, config::Config, draw_state::DrawState, view::View};
use anyhow::Error as AnyhowError;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use derive_more::Constructor;
use mkutils::{ActiveVec, KeyMap, KeyMapState, Output, PointU16, Socket, Terminal, Utils};
use std::io::Error as IoError;
use ulid::Ulid;

#[derive(Constructor)]
pub struct WindowCrosstermEvent {
    pub window_id: Ulid,
    pub crossterm_event: Output<CrosstermEvent, AnyhowError>,
}

#[derive(Constructor)]
pub struct WindowOutput {
    pub window_id: Ulid,
    pub output: Output<(), AnyhowError>,
}

pub struct Window {
    window_id: Ulid,
    views: Option<ActiveVec<View>>,
    socket: Socket,
    terminal: Terminal,
    key_map_state: KeyMapState<Command>,
    draw_state: DrawState,
}

impl Window {
    pub fn new(socket: Socket, screen_size: PointU16, key_map: &KeyMap<Command>) -> Result<Self, IoError> {
        let window_id = Ulid::new();
        let views = None;
        let terminal = Terminal::new(screen_size)?;
        let key_map_state = KeyMapState::new(key_map);
        let draw_state = DrawState::default();
        let window = Self {
            window_id,
            views,
            socket,
            terminal,
            key_map_state,
            draw_state,
        };

        window.ok()
    }

    pub const fn window_id(&self) -> Ulid {
        self.window_id
    }

    pub fn add_view(&mut self, view: View) {
        if let Some(views) = &mut self.views {
            let (active_index, _view) = views.push(view);

            views.set_active_index(active_index);
        } else {
            self.views = ActiveVec::new(view).some();
        }
    }

    async fn on_command(
        &mut self,
        _config: &Config,
        _buffer_map: &BufferMap,
        command: Command,
    ) -> Output<(), AnyhowError> {
        match command {
            Command::Quit => Output::EndOk,
        }
    }

    fn on_input(&mut self, key_event: KeyEvent) -> Output<(), AnyhowError> {
        tracing::info!(?key_event).output_ok()
    }

    async fn on_key_event(
        &mut self,
        config: &Config,
        buffer_map: &mut BufferMap,
        key_event: KeyEvent,
    ) -> Output<(), AnyhowError> {
        // TODO: resolve [command.clone()]
        if let Some(command) = self.key_map_state.on_key_event(&config.key_map, key_event) {
            self.on_command(config, buffer_map, command.clone()).await
        } else {
            self.on_input(key_event)
        }
    }

    fn on_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::Down(_mouse_button) => content_view.toggle_locked_to_bottom(),
            MouseEventKind::ScrollDown => content_view.scroll_down(&mouse_event.modifiers.into()),
            MouseEventKind::ScrollUp => content_view.scroll_up(&mouse_event.modifiers.into()),
            MouseEventKind::ScrollLeft => content_view.scroll_left(&mouse_event.modifiers.into()),
            MouseEventKind::ScrollRight => content_view.scroll_right(&mouse_event.modifiers.into()),
            _ignored_mouse_event_kind => {}
        }
    }


    pub async fn on_crossterm_event(
        &mut self,
        config: &Config,
        buffer_map: &mut BufferMap,
        crossterm_event: CrosstermEvent,
    ) -> Output<(), AnyhowError> {
        match crossterm_event {
            CrosstermEvent::Key(key_event) => self.on_key_event(config, buffer_map, key_event).await,
            CrosstermEvent::Mouse(mouse_event) => self.on_mouse_event(mouse_event).into_continue(),
            CrosstermEvent::Resize(num_cols, num_rows) => self.terminal.resize(num_cols, num_rows)?.output_ok(),
            _crossterm_event => ().output_ok(),
        }
    }

    pub async fn next_window_crossterm_event(&mut self) -> WindowCrosstermEvent {
        WindowCrosstermEvent::new(self.window_id, self.socket.recv().await)
    }

    pub async fn on_tick_impl(&mut self, config: &Config, buffer_map: &mut BufferMap) -> Output<(), AnyhowError> {
        if let Some(command) = self.key_map_state.on_tick(&config.key_map) {
            self.on_command(config, buffer_map, command.clone()).await?;
        }

        self.draw_state
            .draw(config, buffer_map, self.views.as_ref(), &mut self.terminal)?
            .send_to(&mut self.socket)
            .await?;

        ().output_ok()
    }

    pub async fn on_tick(&mut self, config: &Config, buffer_map: &mut BufferMap) -> WindowOutput {
        let output = self.on_tick_impl(config, buffer_map).await;

        WindowOutput::new(self.window_id, output)
    }
}
