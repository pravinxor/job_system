use std::error::Error;

use crate::job::{Job, JobData};

struct RenderJob {
    data: JobData,
    /// The rendering data to be processed by execute()
    render_data: Vec<i64>,
}

impl Job for RenderJob {
    fn execute(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut total = 0;
        total += self.render_data.iter().sum::<i64>();
        total += self.render_data.iter().sum::<i64>();

        // The total goes in the first index, because that's how it was implemented in the cpp?
        if let Some(first_data_idx) = self.render_data.first_mut() {
            *first_data_idx = total;
        }

        eprintln!("Job {} has been executed", self.data.id);
        Ok(())
    }

    fn complete_callback(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(sum) = self.render_data.first() {
            eprintln!("Job {} Calculated sum: {}", self.data.id, sum)
        }
        todo!()
    }

    fn get_unique_id(&self) -> usize {
        self.data.id
    }
}
