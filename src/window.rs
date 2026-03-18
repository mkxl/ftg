use crate::{
    buffer_map::BufferMap, command::Command, config::Config, draw_state::DrawState, view::View, view_state::ViewState,
    view_state_set::ViewStateSet,
};
use anyhow::Error as AnyhowError;
use camino::Utf8PathBuf;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use derive_more::Constructor;
use mkutils::{KeyMap, KeyMapState, Output, PointU16, ScrollView, Socket, Terminal, Utils};
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
    view_state_set: ViewStateSet,
    socket: Socket,
    terminal: Terminal,
    key_map_state: KeyMapState<Command>,
    draw_state: DrawState,
}

impl Window {
    pub fn new(socket: Socket, screen_size: PointU16, key_map: &KeyMap<Command>) -> Result<Self, IoError> {
        let window_id = Ulid::new();
        let view_state_set = ViewStateSet::empty();
        let terminal = Terminal::new(screen_size)?;
        let key_map_state = KeyMapState::new(key_map);
        let draw_state = DrawState::default();
        let window = Self {
            window_id,
            view_state_set,
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

    pub fn add_new_view(&mut self, buffer_id: Ulid, filepath: Option<Utf8PathBuf>) {
        let view_state = ViewState::new(buffer_id, filepath);

        self.view_state_set.push(view_state);
    }

    async fn on_command(
        &mut self,
        _config: &Config,
        buffer_map: &mut BufferMap,
        command: Command,
    ) -> Output<(), AnyhowError> {
        match command {
            Command::Quit => Output::EndOk,
            Command::MoveForward => self
                .view_state_set
                .active_view_mut(buffer_map)?
                .as_mut()
                .map(View::move_forward)
                .unit()
                .output_ok(),
            Command::MoveBackward => self
                .view_state_set
                .active_view_mut(buffer_map)?
                .as_mut()
                .map(View::move_backward)
                .unit()
                .output_ok(),
        }
    }

    fn on_input(&mut self, buffer_map: &mut BufferMap, key_event: KeyEvent) -> Output<(), AnyhowError> {
        let Some(mut view) = self.view_state_set.active_view_mut(buffer_map)? else { return ().output_ok() };
        let KeyCode::Char(character) = key_event.code else { return ().output_ok() };

        view.replace(&character.to_string());

        ().output_ok()
    }

    async fn on_key_event(
        &mut self,
        config: &Config,
        buffer_map: &mut BufferMap,
        key_event: KeyEvent,
    ) -> Output<(), AnyhowError> {
        tracing::info!(?key_event);

        // TODO: resolve [command.clone()]
        if let Some(command) = self.key_map_state.on_key_event(&config.key_map, key_event) {
            self.on_command(config, buffer_map, command.clone()).await
        } else {
            self.on_input(buffer_map, key_event)
        }
    }

    fn on_mouse_event(&mut self, mouse_event: MouseEvent, buffer_map: &mut BufferMap) -> Result<(), AnyhowError> {
        let Some(mut active_view) = self.view_state_set.active_view_mut(buffer_map)? else {
            return ().ok();
        };

        match mouse_event.kind {
            MouseEventKind::ScrollDown => active_view.scroll_view_state_mut().scroll_down(1),
            MouseEventKind::ScrollUp => active_view.scroll_view_state_mut().scroll_up(1),
            MouseEventKind::ScrollLeft => active_view.scroll_view_state_mut().scroll_left(1),
            MouseEventKind::ScrollRight => active_view.scroll_view_state_mut().scroll_right(1),
            _ignored_mouse_event_kind => {}
        }

        ().ok()
    }

    pub async fn on_crossterm_event(
        &mut self,
        config: &Config,
        buffer_map: &mut BufferMap,
        crossterm_event: CrosstermEvent,
    ) -> Output<(), AnyhowError> {
        match crossterm_event {
            CrosstermEvent::Key(key_event) => self.on_key_event(config, buffer_map, key_event).await,
            CrosstermEvent::Mouse(mouse_event) => self.on_mouse_event(mouse_event, buffer_map)?.output_ok(),
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
            .draw(config, buffer_map, &mut self.view_state_set, &mut self.terminal)?
            .send_to(&mut self.socket)
            .await?;

        ().output_ok()
    }

    pub async fn on_tick(&mut self, config: &Config, buffer_map: &mut BufferMap) -> WindowOutput {
        let output = self.on_tick_impl(config, buffer_map).await;

        WindowOutput::new(self.window_id, output)
    }
}
