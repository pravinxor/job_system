use std::error::Error;

use job_system::JobSystem;
use jobs::render::RenderJob;
use rand::Rng;

mod job;
mod job_master;
mod job_slave;
mod job_system;
mod jobs;

fn main() -> Result<(), Box<dyn Error>> {
    let mut jobs = Vec::new();
    dbg!("creating jobs");

    for _ in 0..5 {
        let render_data = (0..10000)
            .map(|_| rand::thread_rng().gen_range(0..=10000))
            .collect();
        let job = Box::new(RenderJob::new(1, 0xFFFFFFFFFF, render_data));
        jobs.push(job);
    }

    dbg!("creating system and slaves");

    let mut system = JobSystem::new()?;
    eprintln!("Created job system");

    for n in 0..16 {
        system.create_slave(format!("thread{}", n), 0xFFFFFFFFFF)?;
    }

    dbg!("adding jobs to queue");
    for job in jobs {
        system.queue_job(job)?
    }

    Ok(())
}
