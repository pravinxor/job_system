extern crate petgraph;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

use super::tokenizer::{BrState, Token, Tokenizer};

#[derive(Debug)]
struct ProcessNode {
    name: String,
}

#[derive(Debug)]
pub(crate) struct ExecutionGraph {
    graph: DiGraph<ProcessNode, ()>, // Empty tuple `()` as edge weight
    node_indices: HashMap<String, NodeIndex>, // For quick node lookup
}

impl ExecutionGraph {
    pub fn new() -> Self {
        ExecutionGraph {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
        }
    }

    fn add_edge(&mut self, tokens: &mut Vec<Token>) {
        dbg!(&tokens);
        tokens.clear();
    }

    fn extract_digraph<I>(&mut self, tokens: &mut I) -> Result<(), String>
    where
        I: Iterator<Item = Token>,
    {
        let mut open_braces = 0;
        let mut token_line = Vec::new();
        while let Some(token) = tokens.next() {
            match token {
                Token::Brace(state) => {
                    open_braces += match state {
                        BrState::Open => 1,
                        BrState::Closed => -1,
                    }
                }
                Token::Arrow | Token::Text(_) | Token::Bracket(_) => token_line.push(token),
                Token::Digraph => return Err(format!("Unexpected token: {:?}", token)),
                Token::Semicolon => self.add_edge(&mut token_line),
            }
        }
        Ok(())
    }

    pub fn from_tokens<I>(mut tokens: I) -> Result<Self, String>
    where
        I: Iterator<Item = Token>,
    {
        let mut graph = Self::new();
        while let Some(token) = tokens.next() {
            if token == Token::Digraph {
                graph.extract_digraph(&mut tokens)?;
            }
        }
        Ok(graph)
    }

    pub fn add_path(&mut self, src_name: &str, dest_name: &str) {
        let src_index = self.get_or_create_node(src_name);
        let dest_index = self.get_or_create_node(dest_name);

        // Add edge if it doesn't already exist
        if self.graph.find_edge(src_index, dest_index).is_none() {
            self.graph.add_edge(src_index, dest_index, ());
        }
    }

    fn get_or_create_node(&mut self, name: &str) -> NodeIndex {
        match self.node_indices.get(name) {
            Some(&index) => index,
            None => {
                let node_index = self.graph.add_node(ProcessNode {
                    name: name.to_owned(),
                });
                self.node_indices.insert(name.to_owned(), node_index);
                node_index
            }
        }
    }

    // Additional methods like graph traversal, execution logic, etc.
}
