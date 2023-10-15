use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

struct MessageQueue<T> {
    queue: Mutex<VecDeque<T>>,
    available: Condvar,
}

impl<T> MessageQueue<T> {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            queue: Mutex::new(VecDeque::new()),
            available: Condvar::new(),
        })
    }
    fn push(&self, value: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(value);
        self.available.notify_one();
    }

    /// Pops element from queue. If multiple threads are waiting on pop(), the thread chosen is nondeterministic
    fn pop(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        // The purpose of the loop is to handle cases of spurious unlocks, where the `available` was notified spuriously
        loop {
            if let Some(value) = queue.pop_front() {
                return value;
            } else {
                queue = self.available.wait(queue).unwrap();
            }
        }
    }
}
