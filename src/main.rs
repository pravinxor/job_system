mod flowscript;
mod jobs;
mod system;

use flowscript::tokenizer::Token;
use jobs::filereader;
use serde_json::json;

use crate::{
    flowscript::execution_graph::ExecutionGraph,
    flowscript::tokenizer::TokenizerAdapter,
    flowscript::util::SpliteratorAdapter,
    jobs::{clangoutput, make},
    system::job_system::JobSystem,
};
use std::{
    error::Error,
    io::{self, Read},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let tokens = input.into_bytes().into_iter().map(|b| b as char).tokens();

    let tokens: Vec<_> = tokens.collect();
    let mut tokens = tokens.into_iter().peekable();

    let mut graph = ExecutionGraph::from_tokens(&mut tokens)?;

    let res = graph.execute_all();
    Ok(())
}
