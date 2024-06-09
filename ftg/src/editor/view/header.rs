use crate::utils::any::Any;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub struct Filepath {
    path: PathBuf,
    string: String,
}

impl Filepath {
    pub fn new(path: PathBuf) -> Self {
        let string = path.display().to_string();

        Self { path, string }
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn name(&self) -> &str {
        if let Some(name) = self.path.file_name().and_then(OsStr::to_str) {
            return name;
        }

        let idx = self.string.rfind(std::path::MAIN_SEPARATOR).unwrap_or(0);

        // TODO: do something w graphemes instead
        &self.string[idx..]
    }
}

pub struct Header {
    filepath: Option<Filepath>,
}

impl Header {
    const DEFAULT_TITLE: &'static str = "Untitled";

    pub fn new(filepath: Option<PathBuf>) -> Self {
        let filepath = filepath.map(Filepath::new);

        Self { filepath }
    }

    pub fn path(&self) -> Option<&Path> {
        self.filepath.as_ref()?.as_path().some()
    }

    pub fn name(&self) -> &str {
        self.filepath.as_ref().map_or(Self::DEFAULT_TITLE, Filepath::name)
    }

    pub fn title(&self) -> &str {
        self.filepath.as_ref().map_or(Self::DEFAULT_TITLE, Filepath::as_str)
    }
}
