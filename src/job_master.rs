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
    job_system::{HistoryEntry, JobStatus},
};

pub enum MasterMessage {
    AddJob(Box<dyn Job>),
    RecvCompletedJob(Result<Box<dyn Job>, Box<dyn Error + Send + Sync>>),
    StopRequest,
}

struct JobMasterThread {
    pub slave_threads: Arc<Mutex<Vec<JobSlave>>>,
    pub running_ids: Arc<Mutex<VecDeque<usize>>>,
    pub completed_jobs: Arc<Mutex<Vec<Box<dyn Job>>>>,
    pub history: Arc<Mutex<Vec<HistoryEntry>>>,
    rx: Receiver<MasterMessage>,
}

impl JobMasterThread {
    fn work(&mut self) {
        while let Ok(message) = self.rx.recv() {
            match message {
                MasterMessage::AddJob(job) => {
                    eprintln!("Master recieved add job {}!", job.get_unique_id());
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
                                &slave_thread.name, e
                            )
                        } else {
                            self.running_ids.lock().as_mut().unwrap().push_back(id);
                        }
                        eprintln!("Submit job");
                    } else {
                        eprintln!(
                            "Warning! Dropped job {} due to absense of slave threads!",
                            id
                        )
                    }
                }
                MasterMessage::RecvCompletedJob(jobstatus) => {
                    eprintln!("master finished ");
                    match jobstatus {
                        Ok(job) => {
                            eprintln!("Master recieved finished job {}!", job.get_unique_id());
                            let recvd_id = job.get_unique_id();
                            self.running_ids
                                .lock()
                                .as_mut()
                                .unwrap()
                                .retain(|id| *id != recvd_id);
                            self.history.lock().as_mut().unwrap().push(HistoryEntry {
                                r#type: job.get_type(),
                                status: JobStatus::JobStatusCompleted,
                            });
                            self.completed_jobs.lock().as_mut().unwrap().push(job);
                        }
                        Err(e) => eprintln!("Receieved Job Error: {}", e),
                    }
                }
                MasterMessage::StopRequest => {
                    let mut slaves = self.slave_threads.lock().unwrap();
                    slaves.clear();

                    break;
                }
            }
        }
    }
}

pub struct JobMaster {
    handle: Option<thread::JoinHandle<()>>,
    pub tx: Sender<MasterMessage>,
}

impl JobMaster {
    pub fn new(
        slave_threads: Arc<Mutex<Vec<JobSlave>>>,
        running_ids: Arc<Mutex<VecDeque<usize>>>,
        completed_jobs: Arc<Mutex<Vec<Box<dyn Job>>>>,
        history: Arc<Mutex<Vec<HistoryEntry>>>,
    ) -> Result<Self, std::io::Error> {
        let (tx, rx) = channel();
        let slave_threads_master = slave_threads.clone();

        let handle = thread::Builder::new()
            .name(String::from("Master"))
            .spawn(|| {
                let mut master = JobMasterThread {
                    slave_threads: slave_threads_master,
                    running_ids,
                    completed_jobs,
                    history,
                    rx,
                };
                master.work();
            })?;

        Ok(Self {
            handle: Some(handle),
            tx,
        })
    }
}

impl Drop for JobMaster {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            if let Err(e) = self.tx.send(MasterMessage::StopRequest) {
                eprintln!("Couldn't send stop request to master thread: {}", e)
            }
            if let Err(e) = handle.join() {
                eprintln!("Couldn't join the master thread: {:?}", e)
            }
        }
    }
}
