extern crate petgraph;
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use serde_json::{json, Value};
use std::{collections::HashMap, error::Error, iter::Peekable, sync::Arc};

use crate::system::job_system::JobSystem;

use super::{
    tokenizer::{BrState, Key, Token},
    util::SpliteratorAdapter,
};

#[derive(Debug, Clone)]
struct ProcessNode {
    name: String,
    attributes: HashMap<Key, String>,
}

#[derive(Debug)]
struct ExecuteArgs(Value, NodeIndex, Arc<DiGraph<ProcessNode, usize>>);

impl ProcessNode {
    pub fn execute(args: ExecuteArgs) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let x = args.0;
        let index = args.1;
        let graph = args.2;

        let pnode = &graph[index];
        match crate::system::job_system::ffi::map_job_identifier(&pnode.name) {
            Some(f) => {
                let y = f(x);
                let res = y["result"]
                    .as_object()
                    .ok_or(format!("Invalid JSON Schema (missing result): {}", y))?;
                let status = y["status"]
                    .as_u64()
                    .ok_or(format!("Invalid JSON Schema (missing status): {}", y))?;

                let mut outgoing_edges: Vec<_> =
                    graph.edges_directed(index, Direction::Outgoing).collect();

                // Sort edges based on their weight (order)
                outgoing_edges.sort_unstable_by(|a, b| a.weight().cmp(b.weight()));

                let next_idx = outgoing_edges
                    .get(status as usize)
                    .and_then(|e| Some(e.target()));
                if let Some(next_idx) = next_idx {
                    let x = json!({"input": res});
                    let args = ExecuteArgs(x, next_idx, graph.to_owned());

                    Self::execute(args)
                } else {
                    Ok(y)
                }
            }
            None => Err(format!("Job name: {} is not registered", &pnode.name).into()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ExecutionGraph {
    name: Option<String>,
    graph: DiGraph<ProcessNode, usize>, // Empty tuple `()` as edge weight
    node_indices: HashMap<String, NodeIndex>, // For quick node lookup
    system: JobSystem<ExecuteArgs, Result<Value, Box<dyn Error + Send + Sync>>>,
    edge_counter: usize,
}

impl ExecutionGraph {
    pub fn new(name: Option<String>, n_threads: usize) -> Self {
        let mut system = JobSystem::new();
        (0..n_threads).for_each(|_| system.add_worker());
        ExecutionGraph {
            name,
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            system,
            edge_counter: 0,
        }
    }

    pub fn execute_all(&mut self) -> Vec<Result<Value, Box<dyn Error + Send + Sync>>> {
        let roots = self.graph.node_indices().filter(|a| {
            self.graph
                .neighbors_directed(*a, Direction::Incoming)
                .count()
                == 0
        });

        let temp_graph = Arc::new(self.graph.to_owned());

        let root_handles: Vec<_> = roots
            .map(|i| {
                let input = match serde_json::from_str(
                    self.graph[i]
                        .attributes
                        .get(&Key::Data)
                        .unwrap_or(&String::default())
                        .as_str(),
                ) {
                    Ok(j) => j,
                    Err(e) => return Err(e),
                };
                Ok(self.system.send_job(
                    ExecuteArgs(input, i, temp_graph.clone()),
                    ProcessNode::execute,
                ))
            })
            .flatten()
            .collect();

        root_handles.into_iter().map(|h| h.get()).collect()
    }

    fn parse_node_attributes(
        &mut self,
        node_name: &str,
        attrs_tokens: &[Token],
    ) -> Result<(), Box<dyn Error>> {
        let node_index = self.get_or_create_node(node_name);

        let mut attributes = HashMap::new();
        let mut iter = attrs_tokens.iter().peekable();

        while let Some(token) = iter.next() {
            if let Token::ReservedText(key) = token {
                if iter.next() == Some(&Token::Equals) {
                    if let Token::Text(value) = iter
                        .next()
                        .ok_or(format!("Expected text after '=' in {:?}", attrs_tokens))?
                    {
                        attributes.insert(key.clone(), value.clone());
                    }
                } else {
                    return Err(format!("Expected '=' after key in {:?}", attrs_tokens).into());
                }
            }
        }

        if let Some(node) = self.graph.node_weight_mut(node_index) {
            node.attributes = attributes;
        }

        Ok(())
    }

    fn parse_line(&mut self, tokens: Vec<Token>) -> Result<(), Box<dyn Error>> {
        match tokens.as_slice() {
            [Token::Text(node_name), Token::Bracket(BrState::Open), ..] => {
                // Handling node attributes
                self.parse_node_attributes(node_name, &tokens[2..tokens.len() - 1])
            }
            [Token::Text(src_name), Token::Arrow, Token::Text(dest_name)] => {
                // Handling directed edges
                self.add_path(src_name, dest_name);
                Ok(())
            }
            [Token::Brace(BrState::Closed)] => Ok(()),
            _ => Err(format!("Unexpected token sequence: {:?}", tokens).into()),
        }
    }

    pub fn from_tokens<I>(tokens: &mut Peekable<I>) -> Result<Self, Box<dyn Error>>
    where
        I: Iterator<Item = Token>,
    {
        tokens
            .next()
            .ok_or("Expect digraph token at beginning of parse")?;
        let graph_name;
        match tokens.next().ok_or("Expected token after digraph")? {
            Token::Brace(BrState::Open) => graph_name = None,
            Token::Text(name) => graph_name = Some(name),
            t => return Err(format!("Unexpected token after digraph: {:?}", t).into()),
        }

        let mut graph = Self::new(graph_name, num_cpus::get());

        let statement_lines = tokens.split_by(|t| *t == Token::Semicolon);
        for line in statement_lines {
            graph.parse_line(line)?;
        }

        Ok(graph)
    }

    pub fn add_path(&mut self, src_name: &str, dest_name: &str) {
        let src_index = self.get_or_create_node(src_name);
        let dest_index = self.get_or_create_node(dest_name);

        // Add edge if it doesn't already exist
        if self.graph.find_edge(src_index, dest_index).is_none() {
            self.graph
                .add_edge(src_index, dest_index, self.edge_counter);
            self.edge_counter += 1;
        }
    }

    fn get_or_create_node(&mut self, name: &str) -> NodeIndex {
        match self.node_indices.get(name) {
            Some(&index) => index,
            None => {
                let node_index = self.graph.add_node(ProcessNode {
                    name: name.to_owned(),
                    attributes: HashMap::new(),
                });
                self.node_indices.insert(name.to_owned(), node_index);
                node_index
            }
        }
    }
}
