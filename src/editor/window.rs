use crate::{editor::view::view::View, utils::container::Identifiable};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ulid::Ulid;

#[derive(Debug, Deserialize, Serialize)]
pub struct WindowArgs {
    pub size: (u16, u16),
    pub filepath: Option<PathBuf>,
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
