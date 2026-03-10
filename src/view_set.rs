use crate::{buffer::Buffer, buffer_map::BufferMap, view::View};
use anyhow::Error as AnyhowError;
use derive_more::From;
use mkutils::{ActiveVec, PointUsize, Utils};
use std::{iter::Flatten, option::IterMut as OptionIterMut, slice::IterMut as SliceIterMut};

#[derive(From)]
pub struct ViewSet {
    views: Option<ActiveVec<View>>,
}

impl ViewSet {
    pub fn empty() -> Self {
        None.into()
    }

    pub fn active_view(&self) -> Option<&View> {
        self.views.as_ref()?.active().some()
    }

    pub fn active_view_mut(&mut self) -> Option<&mut View> {
        self.views.as_mut()?.active_mut().some()
    }

    pub fn active_pair<'a>(&'a self, buffer_map: &'a BufferMap) -> Result<Option<(&'a View, &'a Buffer)>, AnyhowError> {
        let Some(active_view) = self.active_view() else { return None.ok() };
        let buffer = buffer_map.get(&active_view.buffer_id()).check_present()?;

        active_view.pair(buffer).some().ok()
    }

    pub fn active_pair_mut<'a>(
        &'a mut self,
        buffer_map: &'a BufferMap,
    ) -> Result<Option<(&'a mut View, &'a Buffer)>, AnyhowError> {
        let Some(active_view) = self.active_view_mut() else { return None.ok() };
        let buffer = buffer_map.get(&active_view.buffer_id()).check_present()?;

        active_view.pair(buffer).some().ok()
    }

    pub fn add_view(&mut self, view: View) {
        if let Some(views) = &mut self.views {
            let (active_index, _view) = views.push(view);

            views.set_active_index(active_index);
        } else {
            self.views = ActiveVec::new(view).some();
        }
    }
}
