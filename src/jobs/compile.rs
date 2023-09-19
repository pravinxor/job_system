use std::{
    error::Error,
    process::{Command, ExitStatus},
};

use crate::job::{Job, JobData};

#[derive(Debug)]
struct CompileJob {
    data: JobData,
    /// The output buffer containing the data from the process spawned within execute()
    output: String,
    /// The exit code of the process spawned within execute(), only available when the process has finished execution
    return_code: Option<ExitStatus>,
}

impl Job for CompileJob {
    fn execute(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let command = "make automated 2>&1";
        let process = Command::new("/bin/sh").arg("-c").arg(command).spawn()?;
        eprintln!("Job {} has been executed", self.data.id);

        let output = process.wait_with_output()?;
        self.output = String::from_utf8(output.stdout)?;
        self.return_code = Some(output.status);
        Ok(())
    }

    fn complete_callback(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(code) = self.return_code {
            eprintln!("Compile job {} return code {}", self.data.id, code);
        }
        eprintln!("Compile job {} output {}", self.data.id, &self.output);
        Ok(())
    }

    fn get_unique_id(&self) -> usize {
        self.data.id
    }

    fn get_type(&self) -> usize {
        self.data.r#type
    }
}
