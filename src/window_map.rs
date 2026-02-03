use crate::{
    buffer_map::BufferMap,
    config::Config,
    window::{Window, WindowCrosstermEvent, WindowOutput},
};
use derive_more::Constructor;
use mkutils::Utils;
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Constructor)]
pub struct WindowMap {
    window_map: HashMap<Ulid, Window>,
}

impl WindowMap {
    pub fn empty() -> Self {
        Self::new(HashMap::new())
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
        // NOTE: [StreamExt::then()] doesn't work here: [https://chatgpt.com/c/697fe763-fcb8-832f-ae90-59309bf933b0]
        let mut window_outputs = Vec::new();

        for window in self.window_map.values_mut() {
            window.on_tick(config, buffer_map).await.push_to(&mut window_outputs);
        }

        window_outputs
    }
}
