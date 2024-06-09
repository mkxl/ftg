use crate::{
    editor::view::view::{View, ViewContext},
    utils::{any::Any, container::Identifiable},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ulid::Ulid;

#[derive(Debug, Deserialize, Serialize)]
pub struct WindowArgs {
    size: (u16, u16),
    paths: Vec<PathBuf>,
}

impl WindowArgs {
    pub fn new(size: (u16, u16), current_dirpath: &Path, mut paths: Vec<PathBuf>) -> Self {
        for path in &mut paths {
            *path = current_dirpath.join(path.immutable());
        }

        Self { size, paths }
    }

    pub fn size(&self) -> &(u16, u16) {
        &self.size
    }

    pub fn into_paths(self) -> Vec<PathBuf> {
        self.paths
    }
}

pub struct Window {
    id: Ulid,
    views: Vec<View>,
    active_view_index: usize,
}

impl Window {
    pub fn new(views: Vec<View>) -> Self {
        let id = Ulid::new();
        let active_view_index = 0;

        Self {
            id,
            views,
            active_view_index,
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn active_view(&mut self) -> (&mut View, ViewContext) {
        let num_views = self.views.len();
        let (before, view, after) = self.views.split3(self.active_view_index);
        let layout = ViewContext {
            before,
            after,
            index: self.active_view_index,
            num_views,
        };

        (view, layout)
    }
}

impl Identifiable for Window {
    fn id(&self) -> Ulid {
        self.id()
    }
}
