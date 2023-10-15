use std::sync::{Arc, Condvar, Mutex};

use serde::Serialize;

/// A handle that is returned after the system takes a job
pub struct JobHandle<T: Serialize> {
    result: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T: Serialize> JobHandle<T> {
    /// Blocks the current thread until the result is available
    pub fn get(self) -> T {
        todo!()
    }
}
