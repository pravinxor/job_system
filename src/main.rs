mod flowscript;
mod jobs;
mod system;

use clap::Parser;
use serde_json::{json, Value};

use crate::{
    flowscript::execution_graph::ExecutionGraph, flowscript::tokenizer::TokenizerAdapter,
    system::job_system::JobSystem,
};
use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::{self, Read},
};

#[derive(Parser)]
#[clap(version = "1.0", author = "Pravin Ramana")]
struct Args {
    files: Vec<String>,
}

fn print_if_err<T, E>(r: Result<T, E>) -> Result<T, E>
where
    E: Sized + Display,
{
    if let Err(ref e) = r {
        eprintln!("{}", e)
    }
    r
}

fn main() -> Result<(), Box<dyn Error>> {
    main_cli()
}

fn main_cli() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut parser_system = JobSystem::new();
    (0..args.files.len()).for_each(|_| parser_system.add_worker());

    let code_files = args
        .files
        .into_iter()
        .map(|name| File::open(name))
        .flat_map(|r| print_if_err(r))
        .map(|f| -> io::Result<String> {
            let mut f = f;
            let mut data = String::new();
            f.read_to_string(&mut data)?;
            Ok(data)
        })
        .flat_map(|r| print_if_err(r));

    let graph_handles: Vec<_> = code_files
        .into_iter()
        .map(|code| {
            parser_system.send_job(code, |code| {
                let mut tokens = code
                    .into_bytes()
                    .into_iter()
                    .map(|b| b as char)
                    .tokens()
                    .peekable();
                ExecutionGraph::from_tokens(&mut tokens)
            })
        })
        .collect();

    let mut merged_graph: ExecutionGraph = graph_handles
        .into_iter()
        .map(|h| h.get())
        .flat_map(|r| print_if_err(r))
        .sum();

    let res = merged_graph.execute_all();
    // dbg!(res);
    Ok(())
}
