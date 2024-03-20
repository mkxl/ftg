use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

pub struct Lock<T>(Arc<Mutex<T>>);

impl<T> Lock<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    pub async fn get(&self) -> MutexGuard<T> {
        self.0.lock().await
    }
}

// NOTE:
// - #[derive(Clone)] requires T: Clone which is not desired so we implement it manually
// - see [https://doc.rust-lang.org/stable/std/clone/trait.Clone.html#derivable]
impl<T> Clone for Lock<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
