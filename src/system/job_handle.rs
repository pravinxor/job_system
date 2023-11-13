use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone, Debug)]
pub enum Status {
    Queued,
    Running,
    Completed,
}
#[derive(Debug)]
pub(crate) struct HandleInner<X, Y> {
    pub(crate) x: Mutex<Option<X>>,
    pub(crate) f: fn(X) -> Y,
    pub(crate) status: Mutex<Status>,
    pub(crate) result: Mutex<Option<Y>>,
    pub(crate) available: Condvar,
}

/// A handle that is returned after the system takes a job
#[derive(Debug)]
pub struct JobHandle<X, Y> {
    pub(crate) handle_inner: Arc<HandleInner<X, Y>>,
}

impl<X, Y> JobHandle<X, Y> {
    pub(crate) fn new(x: X, f: fn(X) -> Y) -> Self {
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
    pub fn get(self) -> Y {
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
