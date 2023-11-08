mod flowscript;
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
    let tokens = flowscript::tokenizer::Tokenizer::new(
        r#"
            digraph {
                a->b;
                b -> c;
            }
            "#,
    );

    for token in tokens {
        dbg!(token);
    }
    Ok(())
}
