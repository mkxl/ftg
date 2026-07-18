use crate::buffer::Buffer;
use anyhow::Error as AnyhowError;
use camino::Utf8Path;
use mkutils::Utils;
use std::{collections::HashMap, os::unix::fs::MetadataExt};
use ulid::Ulid;

#[derive(Default)]
pub struct BufferMap {
    buffer_map: HashMap<Ulid, Buffer>,
    buffer_id_from_inode: HashMap<u64, Ulid>,
}

impl BufferMap {
    const CONTEXT_GET_FILEPATH_METADATA: &str = "get filepath metadata";

    pub fn new() -> Self {
        Self::default()
    }

    fn add_buffer(&mut self, inode: u64, buffer: Buffer) -> &mut Buffer {
        let buffer_id = buffer.buffer_id();

        self.buffer_id_from_inode.insert(inode, buffer_id);

        self.buffer_map.insert_mut(buffer_id, buffer)
    }

    pub fn get(&self, buffer_id: &Ulid) -> Option<&Buffer> {
        self.buffer_map.get(buffer_id)
    }

    pub fn get_mut(&mut self, buffer_id: &Ulid) -> Option<&mut Buffer> {
        self.buffer_map.get_mut(buffer_id)
    }

    pub async fn get_or_insert_from_filepath(&mut self, filepath: &Utf8Path) -> Result<(Ulid, bool), AnyhowError> {
        let inode = filepath
            .metadata_async()
            .await
            .context_path(Self::CONTEXT_GET_FILEPATH_METADATA, filepath)?
            .ino();
        let (buffer, should_create_buffer) = if let Some(buffer_id) = self.buffer_id_from_inode.get(&inode) {
            self.get(buffer_id).check_present()?.pair(false)
        } else {
            self.add_buffer(inode, Buffer::new()).as_immut().pair(true)
        };

        buffer.buffer_id().pair(should_create_buffer).ok()
    }
}
