use crate::{
    job::Job,
    job_master::{JobMaster, MasterMessage},
    job_slave::JobSlave,
};
use std::{
    collections::VecDeque,
    error::Error,
    io,
    sync::{
        mpsc::{channel, Receiver, SendError, Sender},
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
    pub r#type: usize,
    pub status: JobStatus,
}

pub struct JobSystem {
    slave_threads: Arc<Mutex<Vec<JobSlave>>>,

    completed_jobs: Arc<Mutex<Vec<Box<dyn Job>>>>,

    running_ids: Arc<Mutex<VecDeque<usize>>>,

    history: Arc<Mutex<Vec<HistoryEntry>>>,

    master: JobMaster,
}

impl JobSystem {
    pub fn new() -> Result<Self, io::Error> {
        let slave_threads = Arc::new(Mutex::new(Vec::new()));
        let completed_jobs = Arc::new(Mutex::new(Vec::new()));
        let running_ids = Arc::new(Mutex::new(VecDeque::new()));
        let history = Arc::new(Mutex::new(Vec::new()));

        let master = JobMaster::new(
            slave_threads.clone(),
            running_ids.clone(),
            completed_jobs.clone(),
            history.clone(),
        )?;

        Ok(Self {
            slave_threads,
            completed_jobs,
            running_ids,
            history,
            master,
        })
    }

    pub fn create_slave(&mut self, unique_name: String, channels: u64) -> Result<(), io::Error> {
        let slave = JobSlave::new(unique_name, channels, self.master.tx.clone())?;
        self.slave_threads.lock().as_mut().unwrap().push(slave);
        Ok(())
    }

    fn destroy_all_slaves(&mut self) {
        self.slave_threads.lock().as_mut().unwrap().clear();
    }

    pub fn destroy_slave(&mut self, unique_name: &str) {
        self.slave_threads
            .lock()
            .as_mut()
            .unwrap()
            .retain(|s| &s.name != unique_name)
    }

    pub fn queue_job(&mut self, job: Box<dyn Job>) -> Result<(), SendError<MasterMessage>> {
        self.master.tx.send(MasterMessage::AddJob(job))
    }

    pub fn get_job_status(&self, id: usize) -> JobStatus {
        todo!()
    }

    pub fn is_complete(&self, id: usize) -> bool {
        self.completed_jobs
            .lock()
            .unwrap()
            .iter()
            .any(|j| j.get_unique_id() == id)
    }

    fn claim_job(&mut self, worker_flags: usize) -> Option<Box<dyn Job>> {
        todo!()
    }

    pub fn on_job_completed(&mut self, job_just_executed: Box<dyn Job>) {
        todo!()
    }
}

impl Drop for JobSystem {
    fn drop(&mut self) {
        eprintln!("Sending join request");
        self.master.tx.send(MasterMessage::StopRequest).unwrap();
        eprintln!("Sent join request");
    }
}
