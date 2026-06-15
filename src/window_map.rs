use crate::{
    buffer_map::BufferMap,
    config::Config,
    window::{Window, WindowCrosstermEvent, WindowOutput},
};
use mkutils::Utils;
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Default)]
pub struct WindowMap {
    window_map: HashMap<Ulid, Window>,
}

impl WindowMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_mut(&mut self, window_id: &Ulid) -> Option<&mut Window> {
        self.window_map.get_mut(window_id)
    }

    pub fn insert(&mut self, window: Window) {
        self.window_map.insert(window.window_id(), window);
    }

    pub fn remove(&mut self, window_id: &Ulid) {
        self.window_map.remove(window_id);
    }

    pub async fn next_window_crossterm_event(&mut self) -> WindowCrosstermEvent {
        self.window_map
            .values_mut()
            .map(Window::next_window_crossterm_event)
            .select_all()
            .await
    }

    pub async fn on_tick(&mut self, config: &Config, buffer_map: &mut BufferMap) -> Vec<WindowOutput> {
        self.window_map
            .values_mut()
            .stream()
            .then_collect(async |window| window.on_tick(config, buffer_map).await)
            .await
    }
}
