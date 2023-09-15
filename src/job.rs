use std::{
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

pub struct JobData {
    pub id: usize,
    pub r#type: usize,
    pub channels: usize,
}

impl JobData {
    pub fn new(r#type: usize, channels: usize) -> Self {
        Self {
            /// An id unique to each job (automatically incremented every time a new Job is created)
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            /// A numeric value representing the job's type
            r#type,
            /// A bit mask for the channels that a job should be available to
            channels,
        }
    }
}

pub trait Job: Send + Sync {
    /// The initial call, where job begins execution
    fn execute(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    /// Function that is called once Job::execute() has completed (generally used for cleanup)
    fn complete_callback(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    /// Returns an id that should be unique and specific to the current job
    fn get_unique_id(&self) -> usize;
}
