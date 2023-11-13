use std::{sync::Arc, thread};

use super::{
    job_handle::{HandleInner, Status},
    message_queue::MessageQueue,
};

#[derive(Debug)]
pub(crate) enum WorkerMessage<X, Y>
where
    X: Send + Sync,
    Y: Send + Sync,
{
    Handle(Arc<HandleInner<X, Y>>),

    /// Notifies the thread to stop accepting jobs and exit its worker loop
    Join,
}

#[derive(Debug)]
pub(crate) struct Worker {
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub(crate) fn new<X: Send + Sync + 'static, Y: Send + Sync + 'static>(
        message_receiver: Arc<MessageQueue<WorkerMessage<X, Y>>>,
    ) -> Self {
        Self {
            handle: Some(thread::spawn(|| Self::worker_loop(message_receiver))),
        }
    }

    fn worker_loop<X: Send + Sync, Y: Send + Sync>(
        message_receiver: Arc<MessageQueue<WorkerMessage<X, Y>>>,
    ) {
        while let WorkerMessage::Handle(handle) = message_receiver.recv() {
            if let Some(x) = handle.x.lock().unwrap().take() {
                let func = handle.f;
                *handle.status.lock().unwrap() = Status::Running;
                let y = func(x);
                let mut guarded_result = handle.result.lock().unwrap();
                *guarded_result = Some(y);
                *handle.status.lock().unwrap() = Status::Completed;
                handle.available.notify_all();
            }
        }
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
