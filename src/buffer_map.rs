use crate::buffer::Buffer;
use anyhow::Error as AnyhowError;
use camino::Utf8Path;
use derive_more::Constructor;
use mkutils::Utils;
use std::{collections::HashMap, os::unix::fs::MetadataExt};
use ulid::Ulid;

#[derive(Constructor)]
pub struct BufferMap {
    buffer_map: HashMap<Ulid, Buffer>,
    buffer_id_from_inode: HashMap<u64, Ulid>,
}

impl BufferMap {
    const CONTEXT_GET_FILEPATH_METADATA: &str = "get filepath metadata";
    const CONTEXT_READ_FILEPATH_TEXT: &str = "read filepath text";

    pub fn empty() -> Self {
        Self::new(HashMap::new(), HashMap::new())
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

    pub async fn get_or_insert_from_filepath(&mut self, filepath: &Utf8Path) -> Result<Ulid, AnyhowError> {
        let inode = filepath
            .metadata_async()
            .await
            .context_path(Self::CONTEXT_GET_FILEPATH_METADATA, filepath)?
            .ino();
        let buffer = if let Some(buffer_id) = self.buffer_id_from_inode.get(&inode) {
            self.get(buffer_id).check_present()?
        } else {
            let buffer = Buffer::from_filepath(filepath)
                .await
                .context_path(Self::CONTEXT_READ_FILEPATH_TEXT, filepath)?;

            self.add_buffer(inode, buffer)
        };

        buffer.buffer_id().ok()
    }
}
