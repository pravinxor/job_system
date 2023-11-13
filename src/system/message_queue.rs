use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

#[derive(Debug)]
pub(crate) struct MessageQueue<T>
where
    T: Send + Sync,
{
    queue: Mutex<VecDeque<T>>,
    available: Condvar,
}

impl<T: Send + Sync> MessageQueue<T> {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            queue: Mutex::new(VecDeque::new()),
            available: Condvar::new(),
        })
    }
    pub(crate) fn send(&self, value: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(value);
        self.available.notify_one();
    }

    /// Receives an element from queue. If multiple threads are waiting on recv(), the thread chosen is nondeterministic
    pub(crate) fn recv(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        // The purpose of the loop is to handle cases of unlocks where `available` was notified spuriously
        loop {
            if let Some(value) = queue.pop_front() {
                return value;
            } else {
                queue = self.available.wait(queue).unwrap();
            }
        }
    }
}
