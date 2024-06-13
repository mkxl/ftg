use crate::utils::{any::Any, path::Path};
use itertools::Itertools;
use std::collections::HashSet;

#[derive(Default)]
pub struct Project {
    dirpaths: HashSet<Path>,
}

impl Project {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dirpath(&mut self, dirpath: Path) {
        self.dirpaths.insert(dirpath);
    }

    pub fn title(&self) -> Option<String> {
        if self.dirpaths.is_empty() {
            None
        } else {
            self.dirpaths.iter().filter_map(Path::name).join(", ").some()
        }
    }
}
