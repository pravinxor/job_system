use std::{error::Error, sync::atomic::AtomicUsize};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct JobData {
    id: usize,
    r#type: usize,
    channels: usize,
}

impl JobData {
    fn new(r#type: usize, channels: usize) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            r#type,
            channels,
        }
    }
}

trait Job {
    /// The initial call, where job begins execution
    fn execute(&mut self) -> Result<(), Box<dyn Error>>;
    /// Function that is called once Job::execute() has completed (generally used for cleanup)
    fn complete_callback(&mut self) -> Result<(), Box<dyn Error>>;
    /// Returns an id that should be unique and specific to the current job
    fn get_unique_id(&self) -> isize;
}
