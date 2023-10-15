use std::{sync::Arc, thread};

use super::{job_handle::HandleInner, message_queue::MessageQueue};
use serde::Serialize;

enum WorkerMessage<T: Serialize> {
    Handle(Arc<HandleInner<T>>),

    /// Notifies the thread to stop accepting jobs and exit its worker loop
    Join,
}

struct Worker {
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new<T: Serialize + Send + Sync + 'static>(
        message_receiver: Arc<MessageQueue<WorkerMessage<T>>>,
    ) -> Self {
        Self {
            handle: Some(thread::spawn(|| Self::worker_loop(message_receiver))),
        }
    }

    fn worker_loop<T: Serialize>(message_receiver: Arc<MessageQueue<WorkerMessage<T>>>) {
        while let WorkerMessage::Handle(handle) = message_receiver.pop() {}
        todo!()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .unwrap_or_else(|e| eprintln!("Failed to join thread: {:?}", e));
        }
    }
}
