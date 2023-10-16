use std::sync::Arc;

use serde::Serialize;

use super::{
    job_handle::JobHandle,
    message_queue::MessageQueue,
    worker::{Worker, WorkerMessage},
};

pub(crate) struct JobSystem<T: Serialize + Send + Sync + 'static> {
    workers: Vec<Worker>,
    message_queue: Arc<MessageQueue<WorkerMessage<T>>>,
}

impl<T: Serialize + Send + Sync + 'static> JobSystem<T> {
    pub(crate) fn new() -> Self {
        Self {
            message_queue: MessageQueue::new(),
            workers: Vec::new(),
        }
    }

    pub(crate) fn add_worker(&mut self) {
        self.workers.push(Worker::new(self.message_queue.clone()));
    }

    pub(crate) fn send_job(&mut self, x: T, f: fn(T) -> T) -> JobHandle<T> {
        let handle = JobHandle::new(x, f);
        self.message_queue
            .send(WorkerMessage::Handle(handle.handle_inner.clone()));
        handle
    }
}
