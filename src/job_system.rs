use crate::{job::Job, job_slave::JobSlave};
use std::{collections::VecDeque, sync::Arc, thread};

pub enum JobStatus {
    JobStatusNeverSeen,
    JobStatusQueued,
    JobStatusRunning,
    JobStatusCompleted,
    JobStatusRetired,
    NumJobStatuses,
}

struct HistoryEntry {
    r#type: usize,
    status: JobStatus,
}

pub struct JobSystem {
    worker_threads: Vec<JobSlave>,

    queued_jobs: Arc<VecDeque<Box<dyn Job>>>,
    running_jobs: Arc<VecDeque<Box<dyn Job>>>,
    completed_jobs: Arc<Vec<Box<dyn Job>>>,

    history: Arc<Vec<HistoryEntry>>,

    master_handle: thread::JoinHandle<()>,
}

impl JobSystem {
    pub fn new() -> Self {
        let master_handle = thread::spawn(|| {});

        Self {
            worker_threads: Vec::new(),
            queued_jobs: Arc::new(VecDeque::new()),
            running_jobs: Arc::new(VecDeque::new()),
            completed_jobs: Arc::new(Vec::new()),
            history: Arc::new(Vec::new()),
            master_handle,
        }
    }

    pub fn create_slave(
        &mut self,
        unique_name: String,
        channels: u64,
    ) -> Result<(), std::io::Error> {
        let slave = JobSlave::new(unique_name, channels)?;
        self.worker_threads.push(slave);
        Ok(())
    }

    pub fn destroy_slave(&mut self, unique_name: &str) {
        self.worker_threads
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
