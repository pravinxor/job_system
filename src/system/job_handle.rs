use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use serde::Serialize;

#[derive(Clone)]
pub enum Status {
    Queued,
    Running,
    Completed,
}

pub(crate) struct HandleInner<T: Serialize> {
    pub(crate) x: Mutex<Option<T>>,
    pub(crate) f: fn(T) -> T,
    pub(crate) status: Mutex<Status>,
    pub(crate) result: Mutex<Option<T>>,
    pub(crate) available: Condvar,
}

/// A handle that is returned after the system takes a job
pub struct JobHandle<T: Serialize> {
    pub(crate) handle_inner: Arc<HandleInner<T>>,
}

impl<T: Serialize> JobHandle<T> {
    pub(crate) fn new(x: T, f: fn(T) -> T) -> Self {
        let handle_inner = HandleInner {
            x: Mutex::new(Some(x)),
            f,
            result: Mutex::new(None),
            available: Condvar::new(),
            status: Mutex::new(Status::Queued),
        };
        Self {
            handle_inner: Arc::new(handle_inner),
        }
    }
    /// Consumes the JobHandle and blocks the current thread until the result is available
    pub fn get(self) -> T {
        let mut data_guard = self.handle_inner.result.lock().unwrap();
        // Similar to the message_queue, loop until the data is Some, because the condition variable may spuriously wake up
        loop {
            if let Some(data) = data_guard.take() {
                return data;
            } else {
                data_guard = self.handle_inner.available.wait(data_guard).unwrap();
            }
        }
    }

    pub fn get_status(&self) -> Status {
        self.handle_inner.status.lock().unwrap().clone()
    }
}
