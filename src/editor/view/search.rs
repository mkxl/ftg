#[derive(Default)]
pub struct Search {
    query: String,
}

impl Search {
    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn push(&mut self, chr: char) {
        self.query.push(chr);
    }
}
