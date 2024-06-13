use crate::utils::any::Any;
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::path::{Path as StdPath, PathBuf};

#[derive(Deserialize, Eq, From, Hash, PartialEq, Serialize)]
pub struct Path {
    path: PathBuf,
}

impl Path {
    const INVALID_UNICODE_NAME: &'static str = "???";

    pub fn as_str(&self) -> &str {
        self.path.as_os_str().to_str().unwrap_or(Self::INVALID_UNICODE_NAME)
    }

    pub fn is_dir(&self) -> bool {
        self.path.is_dir()
    }

    pub fn name(&self) -> Option<&str> {
        self.path
            .file_name()?
            .to_str()
            .unwrap_or(Self::INVALID_UNICODE_NAME)
            .some()
    }
}

impl AsRef<StdPath> for Path {
    fn as_ref(&self) -> &StdPath {
        self.path.as_ref()
    }
}
