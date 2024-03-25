use crate::{
    error::Error,
    utils::{any::Any, bytes::Bytes},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

pub struct Terminal {
    bytes: Bytes,
    backend: CrosstermBackend<Bytes>,
    buffer_curr: Buffer,
    buffer_prev: Buffer,
}

impl Terminal {
    pub fn new(area: Rect) -> Self {
        let bytes = Bytes::default();
        let backend = CrosstermBackend::new(bytes.clone());
        let buffer_prev = Buffer::empty(area);
        let buffer_curr = buffer_prev.clone();

        Self {
            bytes,
            backend,
            buffer_curr,
            buffer_prev,
        }
    }

    pub fn render_widget<W: Widget>(&mut self, widget: W, area: Rect) {
        widget.render(area, &mut self.buffer_curr);
    }

    pub fn finish(&mut self) -> Result<Vec<u8>, Error> {
        // NOTE:
        // - buffer_prev: what has already been rendered
        // - buffer_curr: what is to be rendered
        let updates = self.buffer_prev.diff(&self.buffer_curr);

        if !updates.is_empty() {
            self.backend.draw(updates.into_iter())?;
        }

        // NOTE:
        // - after drawing the diff, (1) swap buffer_prev and buffer_curr and (2) reset what is to
        //   to be rendered (ie buffer_curr)
        std::mem::swap(&mut self.buffer_prev, &mut self.buffer_curr);
        self.buffer_curr.reset();

        self.bytes.take().ok()
    }

    pub fn resize(&mut self, area: Rect) {
        self.buffer_curr.resize(area);
        self.buffer_prev.resize(area);
    }

    pub fn area(&self) -> Rect {
        *self.buffer_curr.area()
    }
}
