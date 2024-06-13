use crate::{editor::view::header::Filepath, utils::any::Any};
use itertools::Itertools;
use path_clean::PathClean;
use std::{
    collections::HashSet,
    fmt::{Display, Error as FmtError, Formatter},
    path::PathBuf,
};

pub struct Name<'a>(&'a PathBuf);

impl<'a> Display for Name<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        Filepath::new(self.0.clean()).name().fmt(f)
    }
}

#[derive(Default)]
pub struct Project {
    dirpaths: HashSet<PathBuf>,
}

impl Project {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dirpath(&mut self, dirpath: PathBuf) {
        self.dirpaths.insert(dirpath);
    }

    pub fn title(&self) -> Option<String> {
        if self.dirpaths.is_empty() {
            None
        } else {
            self.dirpaths.iter().map(Name).join(", ").some()
        }
    }
}
