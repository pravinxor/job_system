use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{channel, Receiver, SendError, Sender},
        Arc,
    },
    thread,
};

use crate::{job::Job, job_master::MasterMessage};

pub enum SlaveMessage {
    Job(Box<dyn Job>),
    StopRequest,
}

pub struct JobSlave {
    /// Handle to join slave thread when Self is dropped
    handle: Option<thread::JoinHandle<()>>,

    /// Channel to send instructions to slave thread
    pub tx: Sender<SlaveMessage>,

    /// A bitmask of the channels of jobs that may be performed by the thread
    channels: u64,

    /// The unique name associated with the slave thread
    pub name: String,

    /// The number of jobs queued for completion
    queue_len: Arc<AtomicUsize>,
}

impl JobSlave {
    pub fn new(
        name: String,
        channels: u64,
        tx_thread: Sender<MasterMessage>,
    ) -> Result<Self, std::io::Error> {
        let (tx_system, rx_thread) = channel();
        let queue_len = Arc::new(AtomicUsize::new(0));
        let ql_slave = queue_len.clone();

        let handle =
            thread::Builder::new().spawn(move || JobSlave::work(rx_thread, tx_thread, ql_slave))?;

        Ok(Self {
            handle: Some(handle),
            tx: tx_system,
            channels,
            name,
            queue_len,
        })
    }

    pub fn submit(&self, message: SlaveMessage) -> Result<(), SendError<SlaveMessage>> {
        if let SlaveMessage::Job(_) = message {
            self.queue_len.fetch_add(1, Ordering::Relaxed);
        }
        self.tx.send(message)
    }

    /// The number of jobs in queue for the slave
    pub fn pressure(&self) -> usize {
        self.queue_len.load(Ordering::Relaxed)
    }

    fn work(rx: Receiver<SlaveMessage>, tx: Sender<MasterMessage>, queue_len: Arc<AtomicUsize>) {
        while let Ok(message) = rx.recv() {
            match message {
                SlaveMessage::Job(mut job) => {
                    if let Err(e) = tx.send(MasterMessage::RecvCompletedJob(match job.execute() {
                        Ok(_) => Ok(job),
                        Err(e) => Err(e),
                    })) {
                        eprintln!("Failed to send completed job to Master: {}", e)
                    }
                    queue_len.fetch_sub(1, Ordering::Relaxed);
                }
                SlaveMessage::StopRequest => break,
            }
        }
    }
}

impl Drop for JobSlave {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            if let Err(e) = self.tx.send(SlaveMessage::StopRequest) {
                eprintln!("Couldn't send stop request to slave thread: {}", e)
            }
            if let Err(e) = handle.join() {
                eprintln!("Couldn't join the master thread: {:?}", e)
            }
        }
    }
}
