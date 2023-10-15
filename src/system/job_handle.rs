use std::sync::{Arc, Condvar, Mutex};

use serde::Serialize;

pub(crate) struct HandleInner<T: Serialize> {
    data: Mutex<Option<T>>,
    available: Condvar,
}

/// A handle that is returned after the system takes a job
pub struct JobHandle<T: Serialize> {
    result: Arc<HandleInner<T>>,
}

impl<T: Serialize> JobHandle<T> {
    /// Consumes the JobHandle and blocks the current thread until the result is available
    pub fn get(self) -> T {
        let mut data_guard = self.result.data.lock().unwrap();
        // Similar to the message_queue, loop until the data is Some, because the condition variable may spuriously wake up
        loop {
            if let Some(data) = data_guard.take() {
                return data;
            } else {
                data_guard = self.result.available.wait(data_guard).unwrap();
            }
        }
    }
}
