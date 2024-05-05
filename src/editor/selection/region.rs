use derive_more::From;

#[derive(Debug, From)]
pub struct Region {
    begin: usize,
    end: usize,
}

impl Region {
    pub fn begin(&self) -> usize {
        self.begin
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn min(&self) -> usize {
        self.begin.min(self.end)
    }

    pub fn max(&self) -> usize {
        self.begin.max(self.end)
    }

    pub fn len(&self) -> usize {
        self.end.abs_diff(self.begin)
    }

    pub fn is_empty(&self) -> bool {
        self.begin == self.end
    }
}
