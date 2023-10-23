mod jobs;
mod system;

use jobs::filereader;
use serde_json::json;

use crate::{
    jobs::{clangoutput, make},
    system::job_system::JobSystem,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut system = JobSystem::new();
    for _ in 0..num_cpus::get() {
        system.add_worker();
    }

    let handle = system.send_job(json!({"target": "demo"}), make::output);
    let target_json = handle.get();

    let handle = system.send_job(target_json, clangoutput::parse);
    let output_json = handle.get();

    let handle = system.send_job(output_json, filereader::read_context);
    let context_output_json = handle.get();

    println!("{}", context_output_json);
    Ok(())
}
