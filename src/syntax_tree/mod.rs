use anyhow::{Result, anyhow};
use tree_sitter::{Parser, Tree, Node};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::languages::LanguageManager; // Assuming LanguageManager handles parser loading
use log::info;

// This module provides functionality for working with Abstract Syntax Trees (ASTs)
// using Tree-sitter. It allows for parsing code, querying the tree, and navigating.

pub struct SyntaxTreeManager {
    language_manager: Arc<LanguageManager>,
    parsers: Mutex<HashMap<String, Parser>>, // Cache parsers here if memory allows
}

impl SyntaxTreeManager {
    pub fn new(language_manager: Arc<LanguageManager>) -> Self {
        Self {
            language_manager,
            parsers: Mutex::new(HashMap::new()),
        }
    }

    pub async fn init(&self) -> Result<()> {
        info!("Syntax tree manager initialized.");
        // Load grammars, initialize parsers
        Ok(())
    }

    /// Parses a given code string for a specific language and returns its syntax tree.
    pub async fn parse_code(&self, language_name: &str, code: &str) -> Result<Tree> {
        let mut parsers = self.parsers.lock().await;
        let parser = parsers.entry(language_name.to_string()).or_insert_with(|| {
            let mut parser = Parser::new();
            parser.set_language(self.language_manager.get_language(language_name).unwrap()).unwrap();
            parser
        });

        parser.parse(code, None)
            .ok_or_else(|| anyhow!("Failed to parse code for language: {}", language_name))
    }

    /// Queries the syntax tree using a Tree-sitter query string.
    /// Returns a list of matched nodes.
    pub async fn query_tree(&self, tree: &Tree, query_string: &str, language_name: &str, code: &str) -> Result<Vec<Node<'_>>> {
        let language = self.language_manager.get_language(language_name).unwrap();

        let query = tree_sitter::Query::new(language, query_string)?;
        let mut query_cursor = tree_sitter::QueryCursor::new();
        let matches = query_cursor.matches(&query, tree.root_node(), code.as_bytes());

        let mut nodes = Vec::new();
        for m in matches {
            for capture in m.captures {
                nodes.push(capture.node);
            }
        }
        Ok(nodes)
    }

    /// Navigates the syntax tree to find the node at a specific byte offset.
    pub async fn node_at_byte_offset(&self, tree: &Tree, byte_offset: usize) -> Option<Node<'_>> {
        let point = tree_sitter::Point::new(0, byte_offset); // Assuming single line for simplicity
        tree.root_node().descendant_for_byte_range(byte_offset, byte_offset + 1)
    }

    /// Example: Get all function definitions in a code string.
    pub async fn get_function_definitions(&self, language_name: &str, code: &str) -> Result<Vec<String>> {
        let tree = self.parse_code(language_name, code).await?;
        let language = self.language_manager.get_language(language_name).unwrap();

        // This query needs to be specific to the language. Example for Rust:
        let query_string = match language_name {
            "rust" => "(function_item (identifier) @name)",
            "bash" => "(function_definition name: (word) @name)",
            _ => return Err(anyhow!("Function definition query not available for language: {}", language_name)),
        };

        let query = tree_sitter::Query::new(language, query_string)?;
        let mut query_cursor = tree_sitter::QueryCursor::new();
        let matches = query_cursor.matches(&query, tree.root_node(), code.as_bytes());

        let mut function_names = Vec::new();
        for m in matches {
            for capture in m.captures {
                if capture.index == 0 { // Assuming @name is the first capture
                    function_names.push(capture.node.utf8_text(code.as_bytes())?.to_string());
                }
            }
        }
        Ok(function_names)
    }

    /// Parses a code snippet and returns its syntax tree (mock).
    pub async fn parse_code_mock(&self, code: &str, language: &str) -> Result<String> {
        info!("Parsing code for language '{}' (mock): {}", language, code);
        // In a real implementation, use tree-sitter or an LSP client
        Ok(format!("Mock syntax tree for {} code: {}", language, code))
    }

    /// Finds definitions or references in a given code context (mock).
    pub async fn find_definitions(&self, code: &str, line: usize, column: usize) -> Result<Vec<String>> {
        info!("Finding definitions at line {}, col {} (mock)", line, column);
        // Simulate finding a definition
        Ok(vec!["mock_definition_location.rs:10:5".to_string()])
    }
}

pub fn init() {
    info!("syntax_tree module loaded");
}
