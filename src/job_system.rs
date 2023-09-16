use crate::{job::Job, job_master::MasterMessage, job_slave::JobSlave};
use std::{
    collections::VecDeque,
    error::Error,
    io,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
    thread,
    time::Duration,
};

pub enum JobStatus {
    JobStatusNeverSeen,
    JobStatusQueued,
    JobStatusRunning,
    JobStatusCompleted,
    JobStatusRetired,
    NumJobStatuses,
}

pub struct HistoryEntry {
    r#type: usize,
    status: JobStatus,
}

pub struct JobSystem {
    slave_threads: Arc<Mutex<Vec<JobSlave>>>,

    completed_jobs: Arc<Mutex<Vec<Box<dyn Job>>>>,

    history: Arc<RwLock<Vec<HistoryEntry>>>,

    master_handle: thread::JoinHandle<()>,

    /// Channel to send messages to the master thread
    tx: Sender<MasterMessage>,
    /// Channel for master thread to receive messages from
    rx: Receiver<MasterMessage>,
}

impl JobSystem {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let master_handle = thread::spawn(|| {});

        Self {
            slave_threads: Arc::new(Mutex::new(Vec::new())),
            completed_jobs: Arc::new(Mutex::new(Vec::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            master_handle,
            tx,
            rx,
        }
    }

    pub fn create_slave(&mut self, unique_name: String, channels: u64) -> Result<(), io::Error> {
        let slave = JobSlave::new(unique_name, channels, self.tx.clone())?;
        self.slave_threads.lock().as_mut().unwrap().push(slave);
        Ok(())
    }

    pub fn destroy_slave(&mut self, unique_name: &str) {
        self.slave_threads
            .lock()
            .as_mut()
            .unwrap()
            .retain(|s| s.name().map_or(true, |name| name != unique_name))
    }

    pub fn queue_job(&mut self, job: Box<dyn Job>) {
        todo!()
    }

    pub fn get_job_status(&self, id: usize) -> JobStatus {
        todo!()
    }

    pub fn is_complete(&self, id: usize) -> bool {
        todo!()
    }

    fn claim_job(&mut self, worker_flags: usize) -> Option<Box<dyn Job>> {
        todo!()
    }

    pub fn on_job_completed(&mut self, job_just_executed: Box<dyn Job>) {
        todo!()
    }
}
