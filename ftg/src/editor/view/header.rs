use crate::utils::path::Path;

pub struct Header {
    filepath: Option<Path>,
}

impl Header {
    const DEFAULT_TITLE: &'static str = "Untitled";

    pub fn new(filepath: Option<Path>) -> Self {
        Self { filepath }
    }

    pub fn path(&self) -> Option<&Path> {
        self.filepath.as_ref()
    }

    pub fn name(&self) -> &str {
        // NOTE: calling Path::name() on the wrapped Path should always return a Some-variant Option
        self.filepath
            .as_ref()
            .and_then(Path::name)
            .unwrap_or(Self::DEFAULT_TITLE)
    }

    pub fn title(&self) -> &str {
        self.filepath.as_ref().map_or(Self::DEFAULT_TITLE, Path::as_str)
    }
}
