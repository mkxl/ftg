use crate::{
    editor::view::view::View,
    utils::{any::Any, container::Identifiable},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ulid::Ulid;

#[derive(Debug, Deserialize, Serialize)]
pub struct WindowArgs {
    size: (u16, u16),
    filepath: Option<PathBuf>,
}

impl WindowArgs {
    pub fn new(size: (u16, u16), current_dirpath: &Path, filepath: Option<PathBuf>) -> Self {
        let filepath = Self::get_filepath(current_dirpath, filepath);

        Self { size, filepath }
    }

    fn get_filepath(current_dirpath: &Path, filepath: Option<PathBuf>) -> Option<PathBuf> {
        let filepath = filepath?;
        let filepath = if filepath.is_absolute() {
            filepath
        } else {
            current_dirpath.join(filepath)
        };

        filepath.some()
    }

    pub fn size(&self) -> &(u16, u16) {
        &self.size
    }

    pub fn filepath(&self) -> Option<&Path> {
        self.filepath.as_deref()
    }
}

pub struct Window {
    id: Ulid,
    view_id: Ulid,
}

impl Window {
    pub fn new(view: &View) -> Self {
        let id = Ulid::new();
        let view_id = view.id();

        Self { id, view_id }
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn primary_view_id(&self) -> Ulid {
        self.view_id
    }
}

impl Identifiable for Window {
    fn id(&self) -> Ulid {
        self.id()
    }
}
