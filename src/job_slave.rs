use std::{
    error::Error,
    marker::PhantomData,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use crate::job::Job;

pub enum SystemMessages {
    Job(Box<dyn Job>),
    StopRequest,
}

pub struct JobSlave {
    /// Handle to join slave thread when Self is dropped
    handle: Option<thread::JoinHandle<()>>,

    /// Channel to send instructions to slave thread
    tx: Sender<SystemMessages>,

    /// Channel to recieve results from slave thread
    rx: Receiver<Result<Box<dyn Job>, Box<dyn Error + Send>>>,

    /// A bitmask of the channels of jobs that may be performed by the thread
    channels: u64,
}

impl JobSlave {
    pub fn new(unique_name: String, channels: u64) -> Result<Self, std::io::Error> {
        let (tx_system, rx_thread) = channel();
        let (tx_thread, rx_system) = channel();

        let handle = thread::Builder::new()
            .name(unique_name)
            .spawn(|| JobSlave::work(rx_thread, tx_thread))?;

        Ok(Self {
            handle: Some(handle),
            tx: tx_system,
            rx: rx_system,
            channels,
        })
    }
    pub fn name(&self) -> Option<&str> {
        self.handle.as_ref()?.thread().name()
    }

    fn work(rx: Receiver<SystemMessages>, tx: Sender<Result<Box<dyn Job>, Box<dyn Error + Send>>>) {
        loop {
            match rx.recv() {
                Ok(message) => match message {
                    SystemMessages::Job(mut job) => {
                        if let Err(e) = job.execute() {
                            tx.send(Err(e)).unwrap()
                        }
                    }
                    SystemMessages::StopRequest => break,
                },
                Err(e) => tx.send(Err(Box::new(e))).unwrap(),
            }
        }
    }
}

impl Drop for JobSlave {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}
