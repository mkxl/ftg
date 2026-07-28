use anyhow::Error as AnyhowError;
use mkutils::Rope;
use std::{future::Future, pin::Pin};
use ulid::Ulid;

pub type OperationFuture = Pin<Box<dyn 'static + Future<Output = Result<Operation, AnyhowError>> + Send + Sync>>;

pub enum Operation {
    LoadBuffer { buffer_id: Ulid, rope: Rope },
}
