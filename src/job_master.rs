use std::{
    collections::VecDeque,
    error::Error,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use crate::{
    job::Job,
    job_slave::{JobSlave, SlaveMessage},
    job_system::HistoryEntry,
};

pub enum MasterMessage {
    AddJob(Box<dyn Job>),
    RecvCompletedJob(Result<Box<dyn Job>, Box<dyn Error + Send + Sync>>),
}

struct JobMasterThread {
    pub slave_threads: Arc<Mutex<Vec<JobSlave>>>,
    pub running_ids: Arc<Mutex<VecDeque<usize>>>,
    pub completed_jobs: Arc<Mutex<Vec<Box<dyn Job>>>>,
    pub history: Arc<Vec<HistoryEntry>>,
    tx: Sender<Result<Box<dyn Job>, Box<dyn Error + Send + Sync>>>,
    rx: Receiver<MasterMessage>,
}

impl JobMasterThread {
    fn work(&mut self) {
        while let Ok(message) = self.rx.recv() {
            match message {
                MasterMessage::AddJob(job) => {
                    let id = job.get_unique_id();
                    if let Some(slave_thread) = self
                        .slave_threads
                        .lock()
                        .unwrap()
                        .iter()
                        .min_by_key(|s| s.pressure())
                    {
                        if let Err(e) = slave_thread.submit(SlaveMessage::Job(job)) {
                            eprintln!(
                                "Error when submitting job to slave thread {:?}: {}",
                                slave_thread.name().unwrap_or("anonymous"),
                                e
                            )
                        } else {
                            self.running_ids.lock().as_mut().unwrap().push_back(id);
                        }
                    } else {
                        eprintln!(
                            "Warning! Dropped job {} due to absense of slave threads!",
                            id
                        )
                    }
                }
                MasterMessage::RecvCompletedJob(jobstatus) => match jobstatus {
                    Ok(job) => {
                        let recvd_id = job.get_unique_id();
                        self.running_ids
                            .lock()
                            .as_mut()
                            .unwrap()
                            .retain(|id| *id != recvd_id)
                    }
                    Err(e) => eprintln!("Receieved Error: {}", e),
                },
            }
        }
    }
}

pub struct JobMaster {
    handle: Option<thread::JoinHandle<()>>,
    tx: Sender<MasterMessage>,
    rx: Receiver<Result<Box<dyn Job>, Box<dyn Error + Send + Sync>>>,
}

impl JobMaster {
    pub fn new(
        slave_threads: Arc<Mutex<Vec<JobSlave>>>,
        running_ids: Arc<Mutex<VecDeque<usize>>>,
        completed_jobs: Arc<Mutex<Vec<Box<dyn Job>>>>,
        history: Arc<Vec<HistoryEntry>>,
    ) -> Result<Self, std::io::Error> {
        let (tx_system, rx_thread) = channel();
        let (tx_thread, rx_system) = channel();

        let handle = thread::Builder::new()
            .name(String::from("Master"))
            .spawn(|| {
                let mut master = JobMasterThread {
                    slave_threads,
                    running_ids,
                    completed_jobs,
                    history,
                    tx: tx_thread,
                    rx: rx_thread,
                };
                master.work();
            })?;

        Ok(Self {
            handle: Some(handle),
            tx: tx_system,
            rx: rx_system,
        })
    }
}

impl Drop for JobMaster {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}
