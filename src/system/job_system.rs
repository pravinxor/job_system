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
        static ref ID_COUNTER: AtomicU64 = AtomicU64::new(0);
        static ref JOB_MAP: DashMap<u64, JobHandle<Value>> = DashMap::new();
        static ref SYSTEM_MAP: DashMap<u64, Mutex<JobSystem<Value>>> = DashMap::new();
    }

    type JobDef = fn(Value) -> Value;

    fn map_job_identifier(identifier: &str) -> Option<JobDef> {
        match identifier {
            "make" => Some(crate::jobs::make::output),
            "clang_parse" => Some(crate::jobs::clangoutput::parse),
            "add_context" => Some(crate::jobs::filereader::read_context),
            _ => None,
        }
    }
    macro_rules! into_raw_cstr {
        ($json_val:expr) => {{
            let c_str = CString::new($json_val.to_string()).unwrap();
            c_str.into_raw()
        }};
    }
    // macro_rules! from_raw_cstr {
    //     ($raw_cstr:expr) => {{
    //         !raw_cstr.is_null() {
    //             unsafe { Ok(CStr::from_ptr(json_str_ptr).to_str().unwrap()) }
    //         } else {
    //             Err
    //         }
    //     }}
    // }
    macro_rules! parse_json_from_str {
        ($input_str:expr) => {{
            Value::from_str($input_str).map_err(|_| "Unable to parse json")
        }};
    }
    macro_rules! fetch_system_from_json {
        ($job_json:expr) => {{
            let system_id = $job_json["system_id"]
                .as_u64()
                .ok_or("'system_id' key is not a valid number or may not exist")?;

            let system = SYSTEM_MAP
                .get_mut(&system_id)
                .ok_or("Specified system id could not be found");

            system
        }};
    }

    #[no_mangle]
    pub extern "C" fn create_jobsystem() -> *const c_char {
        let system = Mutex::new(JobSystem::new());
        let id = ID_COUNTER.fetch_add(1, Relaxed);
        SYSTEM_MAP.insert(id, system);
        let output_json = json!({"success" : true, "system_id" : id});

        let c_str = CString::new(output_json.to_string()).unwrap();
        c_str.into_raw()
    }

    #[no_mangle]
    pub extern "C" fn add_worker(json_str_ptr: *const c_char) -> *const c_char {
        let output_json = if json_str_ptr.is_null() {
            json!({"error" : "json_str_ptr was a null pointer"})
        } else {
            let input_str = unsafe { CStr::from_ptr(json_str_ptr).to_str().unwrap() };

            match query_system_add_worker(input_str) {
                Ok(()) => json!({"success" : true}),
                Err(message) => json!({"success" : false, "error" : message}),
            }
        };
        into_raw_cstr!(output_json)
    }

    fn query_system_add_worker(input_str: &str) -> Result<(), String> {
        let job_json = parse_json_from_str!(input_str)?;

        let system = fetch_system_from_json!(job_json)?;
        let mut system = system.lock().unwrap();

        system.add_worker();
        Ok(())
    }

    #[no_mangle]
    pub extern "C" fn get_job(json_str_ptr: *const c_char) -> *const c_char {
        assert!(!json_str_ptr.is_null());
        let output_json = if json_str_ptr.is_null() {
            json!({"error" : "json_str_ptr was a null pointer"})
        } else {
            let input_str = unsafe { CStr::from_ptr(json_str_ptr).to_str().unwrap() };

            match process_and_query_job(input_str) {
                Ok(handle_id) => json!({"success" : true, "handle_id" : handle_id}),
                Err(message) => json!({"success" : false, "error" : message}),
            }
        };
        into_raw_cstr!(output_json)
    }

    fn process_and_query_job(input_str: &str) -> Result<Value, String> {
        let job_json = parse_json_from_str!(input_str)?;

        let handle_id = job_json["handle_id"]
            .as_u64()
            .ok_or("'type' handle_id is not a valid number or may not exist")?;

        let handle = JOB_MAP
            .remove(&handle_id)
            .map(|e| e.1)
            .ok_or("specified handle id was not found")?;

        Ok(handle.get())
    }

    #[no_mangle]
    /// Sends the specified command to the JobSystem, given a JSON with key "type", specifying jobtype and "input", specifying the input data for the job.
    pub extern "C" fn send_job(json_str_ptr: *const c_char) -> *const c_char {
        let output_json = if json_str_ptr.is_null() {
            json!({"error" : "json_str_ptr was a null pointer"})
        } else {
            let input_str = unsafe { CStr::from_ptr(json_str_ptr).to_str().unwrap() };

            match process_and_load_job(input_str) {
                Ok(handle_id) => json!({"success" : true, "handle_id" : handle_id}),
                Err(message) => json!({"success" : false, "error" : message}),
            }
        };

        into_raw_cstr!(output_json)
    }

    fn process_and_load_job(input_str: &str) -> Result<u64, String> {
        let job_json = parse_json_from_str!(input_str)?;

        let system = fetch_system_from_json!(job_json)?;

        let job_type = job_json["type"]
            .as_str()
            .ok_or("'type' key is not a string or may not exist")?;

        let job: Option<fn(Value) -> Value> = map_job_identifier(job_type);

        let job_fn = job.ok_or(format!("job type '{}' was not found", job_type))?;

        let id = ID_COUNTER.fetch_add(1, Relaxed);
        let input = job_json["input"].clone();
        let mut system = system.lock().unwrap();
        let handle = system.send_job(input, job_fn);
        JOB_MAP.insert(id, handle);

        Ok(id)
    }

    #[no_mangle]
    pub extern "C" fn free_str(ptr: *mut c_char) {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}
