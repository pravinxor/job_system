extern crate petgraph;
use petgraph::graph::{DiGraph, NodeIndex};
use std::{collections::HashMap, iter::Peekable};

use super::{
    tokenizer::{BrState, Key, Token},
    util::SpliteratorAdapter,
};

#[derive(Debug)]
struct ProcessNode {
    name: String,
    attributes: HashMap<Key, String>,
}

#[derive(Debug)]
pub(crate) struct ExecutionGraph {
    name: Option<String>,
    graph: DiGraph<ProcessNode, ()>, // Empty tuple `()` as edge weight
    node_indices: HashMap<String, NodeIndex>, // For quick node lookup
}

impl ExecutionGraph {
    pub fn new(name: Option<String>) -> Self {
        ExecutionGraph {
            name,
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
        }
    }

    fn parse_node_attributes(
        &mut self,
        node_name: &str,
        attrs_tokens: &[Token],
    ) -> Result<(), String> {
        let node_index = self.get_or_create_node(node_name);

        let mut attributes = HashMap::new();
        let mut i = 0;
        while i < attrs_tokens.len() {
            match (
                attrs_tokens.get(i).take(),
                attrs_tokens.get(i + 1),
                attrs_tokens.get(i + 2).take(),
            ) {
                (Some(Token::ReservedText(key)), Some(Token::Equals), Some(Token::Text(value))) => {
                    attributes.insert(*key, value.to_owned());
                    i += 3;
                }
                _ => return Err(format!("Invalid attribute format in {:?}", attrs_tokens)),
            }
        }

        if let Some(node) = self.graph.node_weight_mut(node_index) {
            node.attributes = attributes;
        }

        Ok(())
    }

    fn parse_line(&mut self, tokens: Vec<Token>) -> Result<(), String> {
        if tokens.is_empty() {
            return Ok(());
        }

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
            _ => Err(format!("Unexpected token sequence: {:?}", tokens)),
        }
    }

    pub fn from_tokens<I>(tokens: &mut Peekable<I>) -> Result<Self, String>
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
            t => return Err(format!("Unexpected token after digraph: {:?}", t)),
        }
        let mut graph = Self::new(graph_name);

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
            self.graph.add_edge(src_index, dest_index, ());
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
