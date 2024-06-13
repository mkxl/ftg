use crate::{
    editor::{
        buffer::buffer::Buffer, color_scheme::ColorScheme, render::Render, terminal::Terminal, view::view::View,
        window::project::Project,
    },
    error::Error,
    utils::{
        any::Any,
        container::{Container, Identifiable},
    },
};
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ulid::Ulid;

#[derive(Debug, Deserialize, Serialize)]
pub struct WindowArgs {
    terminal_shape: (u16, u16),
    paths: Vec<PathBuf>,
}

impl WindowArgs {
    pub fn new(terminal_shape: (u16, u16), current_dirpath: &Path, mut paths: Vec<PathBuf>) -> Self {
        for path in &mut paths {
            *path = current_dirpath.join(path.immutable());
        }

        Self { terminal_shape, paths }
    }

    pub fn terminal_shape(&self) -> &(u16, u16) {
        &self.terminal_shape
    }

    pub fn into_paths(self) -> Vec<PathBuf> {
        self.paths
    }
}

pub struct Window {
    id: Ulid,
    views: Vec<View>,
    active_view_index: usize,
    terminal: Terminal,
    project: Project,
}

impl Window {
    pub fn new(project: Project, views: Vec<View>, terminal_area: Rect) -> Self {
        let id = Ulid::new();
        let active_view_index = 0;
        let terminal = Terminal::new(terminal_area);

        Self {
            id,
            views,
            active_view_index,
            terminal,
            project,
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn resize(&mut self, width: u16, height: u16) -> Result<(), Error> {
        self.terminal.resize((width, height).rect())
    }

    pub fn active_view(&mut self) -> &mut View {
        &mut self.views[self.active_view_index]
    }

    // TODO: come up w a better implementation that doesn't do this casting
    #[allow(clippy::cast_possible_wrap)]
    fn change_view(&mut self, change: isize) {
        self.active_view_index = change
            .saturating_add_unsigned(self.active_view_index)
            .rem_euclid(self.views.len() as isize) as usize;
    }

    pub fn next_view(&mut self) {
        self.change_view(1);
    }

    pub fn previous_view(&mut self) {
        self.change_view(-1);
    }

    pub fn render(&mut self, buffers: &Container<Buffer>, color_scheme: &ColorScheme) -> Result<Vec<u8>, Error> {
        Render::new(
            &mut self.terminal,
            &self.views,
            self.active_view_index,
            &self.project,
            buffers,
            color_scheme,
        )?
        .render()
    }
}

impl Identifiable for Window {
    fn id(&self) -> Ulid {
        self.id()
    }
}
