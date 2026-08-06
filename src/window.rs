use crate::{
    buffer_map::BufferMap, command::Command, config::Config, tabs_view_state::TabsViewState, view_state::ViewState,
    window_render_state::WindowRenderState,
};
use anyhow::Error as AnyhowError;
use camino::Utf8PathBuf;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use derive_more::Constructor;
use mkutils::{KeyMap, KeyMapState, Output, PointU16, ScrollView, Socket, Terminal, Utils};
use std::{io::Error as IoError, time::Instant};
use ulid::Ulid;

macro_rules! execute {
    ($self:expr, $buffer_map:expr, $config:expr, $($expr:tt)*) => {
        $self
            .tabs_view_state
            .active_view_mut($buffer_map, $config)?
            .as_mut()
            .map(|view| view.$($expr)*)
            .unit()
            .output_ok()
    };
}

#[derive(Constructor)]
pub struct WindowCrosstermEvent {
    pub window_id: Ulid,
    pub crossterm_event: Output<CrosstermEvent, AnyhowError>,
}

#[derive(Constructor)]
pub struct WindowOutput {
    pub window_id: Ulid,
    pub output: Output<(), AnyhowError>,
    pub redraw_all_windows: bool,
}

pub struct Window {
    window_id: Ulid,
    socket: Socket,
    terminal: Terminal,
    key_map_state: KeyMapState<Command>,
    tabs_view_state: TabsViewState,
    window_render_state: WindowRenderState,
    pending_terminal_size: Option<PointU16>,
    redraw_requested: bool,
    redraw_deadline: Option<Instant>,
}

impl Window {
    const FORCE_TERMINAL_FULL_REDRAWS: bool = false;

    pub fn new(socket: Socket, screen_size: PointU16, key_map: &KeyMap<Command>) -> Result<Self, IoError> {
        let window_id = Ulid::new();
        let terminal = Terminal::new(screen_size, Self::FORCE_TERMINAL_FULL_REDRAWS)?;
        let key_map_state = KeyMapState::new(key_map);
        let tabs_view_state = TabsViewState::new();
        let window_render_state = WindowRenderState::default();
        let pending_terminal_size = None;
        let redraw_requested = true;
        let redraw_deadline = None;
        let window = Self {
            window_id,
            socket,
            terminal,
            key_map_state,
            tabs_view_state,
            window_render_state,
            pending_terminal_size,
            redraw_requested,
            redraw_deadline,
        };

        window.ok()
    }

    pub const fn window_id(&self) -> Ulid {
        self.window_id
    }

    pub fn push(&mut self, buffer_id: Ulid, absolute_filepath: Option<Utf8PathBuf>) {
        self.tabs_view_state.push(buffer_id, absolute_filepath);
        self.request_redraw();
    }

    pub const fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    fn schedule_scroll_bar_expiry_redraw(&mut self) {
        self.redraw_deadline = ViewState::RENDER_SCROLL_BARS_TIMEOUT.map(|duration| Instant::now() + duration);
    }

    fn redraw_deadline_elapsed(&self, now: Instant) -> bool {
        self.redraw_deadline.is_some_and(|deadline| deadline <= now)
    }

    fn apply_pending_terminal_resize(&mut self) -> Result<(), IoError> {
        let Some(size) = self.pending_terminal_size.take() else { return ().ok() };
        let current_size = self.terminal.size()?;

        if current_size.x != size.x || current_size.y != size.y {
            self.terminal.resize(size.x, size.y)?;
        }

        ().ok()
    }

    #[expect(clippy::unused_async, clippy::unused_async_trait_impl)]
    async fn on_command(
        &mut self,
        config: &Config,
        buffer_map: &mut BufferMap,
        command: Command,
    ) -> Output<(), AnyhowError> {
        let schedule_scroll_bar_expiry_redraw = matches!(
            &command,
            Command::ScrollRight(_) | Command::ScrollLeft(_) | Command::ScrollUp(_) | Command::ScrollDown(_)
        );
        let output = match command {
            Command::Quit => Output::EndOk,
            Command::MoveBackward => execute!(self, buffer_map, config, move_backward()),
            Command::MoveForward => execute!(self, buffer_map, config, move_forward()),
            Command::MoveUp => execute!(self, buffer_map, config, move_up()),
            Command::MoveDown => execute!(self, buffer_map, config, move_down()),
            Command::ScrollRight(count) => {
                execute!(self, buffer_map, config, scroll_view_state_mut().scroll_right(count))
            }
            Command::ScrollLeft(count) => {
                execute!(self, buffer_map, config, scroll_view_state_mut().scroll_left(count))
            }
            Command::ScrollUp(count) => {
                execute!(self, buffer_map, config, scroll_view_state_mut().scroll_up(count))
            }
            Command::ScrollDown(count) => {
                execute!(self, buffer_map, config, scroll_view_state_mut().scroll_down(count))
            }
            Command::Delete => execute!(self, buffer_map, config, delete()),
            Command::ToggleStatusBar => self.window_render_state.toggle_status_bar().output_ok(),
            Command::ToggleGutter => self.window_render_state.toggle_gutter().output_ok(),
            Command::ToggleTabs => self.window_render_state.toggle_tabs().output_ok(),
            Command::ToggleLogs => self.window_render_state.toggle_logs().output_ok(),
            Command::NextTab => self.tabs_view_state.cycle_next().output_ok(),
            Command::PrevTab => self.tabs_view_state.cycle_prev().output_ok(),
        };

        if schedule_scroll_bar_expiry_redraw {
            self.schedule_scroll_bar_expiry_redraw();
        }

        output
    }

    fn on_input(
        &mut self,
        buffer_map: &mut BufferMap,
        config: &Config,
        key_event: KeyEvent,
    ) -> Output<(), AnyhowError> {
        let Some(mut view) = self.tabs_view_state.active_view_mut(buffer_map, config)? else { return ().output_ok() };
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

        // TODO-9c5d7d: resolve [command.clone()]
        if let Some(command) = self.key_map_state.on_key_event(&config.key_map, key_event) {
            self.on_command(config, buffer_map, command.clone()).await
        } else {
            self.on_input(buffer_map, config, key_event)
        }
    }

    fn on_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        buffer_map: &mut BufferMap,
        config: &Config,
    ) -> Result<(), AnyhowError> {
        let position = (mouse_event.column, mouse_event.row).into();
        let latest_window_area_set = self.window_render_state.latest_window_area_set();

        let scroll_target_handled = if latest_window_area_set.tabs.contains(position) {
            self.tabs_view_state.get_tabs_view(config).on_mouse_event(mouse_event);
            true
        } else if latest_window_area_set.view.contains(position)
            && let Some(mut active_view) = self.tabs_view_state.active_view_mut(buffer_map, config)?
        {
            active_view.on_mouse_event(mouse_event);
            true
        } else {
            false
        };

        if scroll_target_handled
            && matches!(
                mouse_event.kind,
                MouseEventKind::ScrollLeft
                    | MouseEventKind::ScrollRight
                    | MouseEventKind::ScrollUp
                    | MouseEventKind::ScrollDown
            )
        {
            self.schedule_scroll_bar_expiry_redraw();
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
            CrosstermEvent::Key(key_event) => {
                let output = self.on_key_event(config, buffer_map, key_event).await;

                self.request_redraw();

                output
            }
            CrosstermEvent::Mouse(mouse_event) => {
                self.on_mouse_event(mouse_event, buffer_map, config)?;
                self.request_redraw();

                ().output_ok()
            }
            CrosstermEvent::Resize(num_cols, num_rows) => {
                self.pending_terminal_size = PointU16::new(num_cols, num_rows).some();
                self.request_redraw();

                ().output_ok()
            }
            _crossterm_event => ().output_ok(),
        }
    }

    pub async fn next_window_crossterm_event(&mut self) -> WindowCrosstermEvent {
        WindowCrosstermEvent::new(self.window_id, self.socket.recv().await)
    }

    pub async fn on_tick_impl(
        &mut self,
        config: &Config,
        buffer_map: &mut BufferMap,
        latest_logs: &[String],
        redraw_all_windows: &mut bool,
    ) -> Output<(), AnyhowError> {
        // TODO-9c5d7d
        if let Some(command) = self.key_map_state.on_tick(&config.key_map) {
            *redraw_all_windows |= matches!(&command, Command::Delete);
            self.on_command(config, buffer_map, command.clone()).await?;
            self.request_redraw();
        }

        self.apply_pending_terminal_resize()?;

        let now = Instant::now();
        let redraw_deadline_elapsed = self.redraw_deadline_elapsed(now);

        if self.redraw_requested || redraw_deadline_elapsed {
            let bytes = self.window_render_state.draw(
                config,
                buffer_map,
                latest_logs,
                self.tabs_view_state.ref_mut(),
                self.terminal.ref_mut(),
            )?;

            if !bytes.is_empty() {
                bytes.send_to(self.socket.ref_mut()).await?;
            }

            self.redraw_requested = false;

            if redraw_deadline_elapsed {
                self.redraw_deadline = None;
            }
        }

        ().output_ok()
    }

    pub async fn on_tick(
        &mut self,
        config: &Config,
        buffer_map: &mut BufferMap,
        latest_logs: &[String],
    ) -> WindowOutput {
        let mut redraw_all_windows = false;
        let output = self
            .on_tick_impl(config, buffer_map, latest_logs, &mut redraw_all_windows)
            .await;

        WindowOutput::new(self.window_id, output, redraw_all_windows)
    }
}
