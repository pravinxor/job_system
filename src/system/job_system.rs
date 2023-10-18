use std::sync::Arc;

use serde::Serialize;

use super::{
    job_handle::JobHandle,
    message_queue::MessageQueue,
    worker::{Worker, WorkerMessage},
};

pub(crate) struct JobSystem<T: Serialize + Send + Sync + 'static> {
    workers: Vec<Worker>,
    message_queue: Arc<MessageQueue<WorkerMessage<T>>>,
}

impl<T: Serialize + Send + Sync + 'static> JobSystem<T> {
    pub(crate) fn new() -> Self {
        Self {
            message_queue: MessageQueue::new(),
            workers: Vec::new(),
        }
    }

    pub(crate) fn add_worker(&mut self) {
        self.workers.push(Worker::new(self.message_queue.clone()));
    }

    pub(crate) fn send_job(&mut self, x: T, f: fn(T) -> T) -> JobHandle<T> {
        let handle = JobHandle::new(x, f);
        self.message_queue
            .send(WorkerMessage::Handle(handle.handle_inner.clone()));
        handle
    }
}

impl<T: Serialize + Send + Sync> Drop for JobSystem<T> {
    fn drop(&mut self) {
        for _ in 0..self.workers.len() {
            self.message_queue.send(WorkerMessage::Join)
        }
    }
}

pub mod ffi {
    use dashmap::DashMap;
    use lazy_static::lazy_static;
    use serde_json::{json, Value};
    use std::{
        ffi::{c_char, CStr, CString},
        str::FromStr,
        sync::{atomic::AtomicUsize, Mutex},
    };

    use crate::system::job_handle::JobHandle;

    use super::JobSystem;

    lazy_static! {
        static ref JOB_MAP: DashMap<usize, JobHandle<Value>> = DashMap::new();
        static ref JOB_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        static ref SYSTEM: Mutex<JobSystem<Value>> = Mutex::new(JobSystem::new());
    }

    #[no_mangle]
    /// Sends the specified command to the JobSystem, given a JSON with key "type", specifying jobtype and "input", specifying the input data for the job.
    pub extern "C" fn send_job(json_str_ptr: *const c_char) -> *const c_char {
        assert!(!json_str_ptr.is_null());
        let input_str = unsafe { CStr::from_ptr(json_str_ptr).to_str().unwrap() };

        let output_json;
        if let Ok(job_json) = Value::from_str(input_str) {
            if let Some(job_type) = job_json["type"].as_str() {
                let job: Option<fn(Value) -> Value> = match job_type {
                    "make" => Some(crate::jobs::make::output),
                    "clang_parse" => Some(crate::jobs::clangoutput::parse),
                    "add_context" => Some(crate::jobs::filereader::read_context),
                    _ => None,
                };
                if let Some(job_fn) = job {
                    let id = JOB_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let input = job_json["input"].clone();
                    let mut system = SYSTEM.lock().unwrap();
                    let handle = system.send_job(input, job_fn);
                    JOB_MAP.insert(id, handle);
                    output_json = json!({"success" : true, "handle_id" : id});
                } else {
                    output_json = json!({"success" : false, "error" : format!("job type '{}' was not found", job_type)})
                }
            } else {
                output_json = json!({"success" : false , "error" : "'type' key is not a string or may not exist"});
            }
        } else {
            output_json = json!({"success" : false, "error" : "unable to parse input, job_json"})
        }
        let c_str = CString::new(output_json.to_string()).unwrap();

        c_str.into_raw()
    }

    pub extern "C" fn free_str() {}
}
