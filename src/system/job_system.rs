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
    use std::sync::atomic::Ordering::Relaxed;
    use std::{
        ffi::{c_char, CStr, CString},
        str::FromStr,
        sync::{atomic::AtomicU64, Mutex},
    };

    use crate::system::job_handle::JobHandle;

    use super::JobSystem;

    lazy_static! {
        static ref JOB_MAP: DashMap<u64, JobHandle<Value>> = DashMap::new();
        static ref ID_COUNTER: AtomicU64 = AtomicU64::new(0);
        static ref SYSTEM_MAP: DashMap<u64, Mutex<JobSystem<Value>>> = DashMap::new();
    }

    #[no_mangle]
    pub extern "C" fn create_jobsystem(json_str_ptr: *const c_char) -> *const c_char {
        let system = Mutex::new(JobSystem::new());
        let id = ID_COUNTER.fetch_add(1, Relaxed);
        SYSTEM_MAP.insert(id, system);
        let output_json = json!({"success" : true, "system_id" : id});

        let c_str = CString::new(output_json.to_string()).unwrap();
        return c_str.into_raw();
    }

    #[no_mangle]
    pub extern "C" fn get_job(json_str_ptr: *const c_char) -> *const c_char {
        assert!(!json_str_ptr.is_null());
        let input_str = unsafe { CStr::from_ptr(json_str_ptr).to_str().unwrap() };

        let output_json;
        if let Ok(job_json) = Value::from_str(input_str) {
            if let Some(handle_id) = job_json["handle_id"].as_u64() {
                if let Some(handle) = JOB_MAP.remove(&handle_id).and_then(|e| Some(e.1)) {
                    let result = handle.get();
                    output_json = json!({"success" : true, "result" : result});
                } else {
                    output_json =
                        json!({"success" : false, "error" : "specified handle id was not found"});
                }
            } else {
                output_json = json!({"success": false,"error" : "'type' handle_id is not an int or may not exist"});
            }
        } else {
            output_json = json!({"success" : false, "error" : "unable to parse input, job_json"})
        }

        let c_str = CString::new(output_json.to_string()).unwrap();

        c_str.into_raw()
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
                    let id = ID_COUNTER.fetch_add(1, Relaxed);
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
