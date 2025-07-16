use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use regex::Regex;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::highlighting::Highlighter;
use syntect::util::LinesWith;
use tree_sitter::{Parser, Language};
use tree_sitter_bash;
use unicode_segmentation::UnicodeSegmentation;
use rand::Rng;
use rand::distributions::{Alphanumeric, DistString};
use crate::string_offset::StringOffsetManager;
use crate::sum_tree::SumTreeManager;
use crate::workflows::manager::WorkflowManager;
use std::sync::Arc;
use anyhow::Result;
use crate::fuzzy_match::FuzzyMatchManager;

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub iterations: u32,
}

#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    pub results: Vec<BenchmarkResult>,
}

impl BenchmarkSuite {
    pub fn get_performance_summary(&self) -> String {
        let mut summary = String::new();
        for res in &self.results {
            summary.push_str(&format!("  {}: {:.2?} ({} iterations)\n", res.name, res.duration, res.iterations));
        }
        summary
    }
}

pub struct PerformanceBenchmarks {
    workflow_manager: Arc<WorkflowManager>,
    fuzzy_match_manager: Arc<FuzzyMatchManager>,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    string_offset_manager: StringOffsetManager,
    sum_tree_manager: SumTreeManager,
}

impl PerformanceBenchmarks {
    pub fn new(workflow_manager: Arc<WorkflowManager>, fuzzy_match_manager: Arc<FuzzyMatchManager>) -> Self {
        Self {
            workflow_manager,
            fuzzy_match_manager,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            string_offset_manager: StringOffsetManager::new(),
            sum_tree_manager: SumTreeManager::new(),
        }
    }

    pub async fn run_all_benchmarks(&mut self) -> BenchmarkSuite {
        let mut results = Vec::new();

        results.push(self.benchmark_file_io().await);
        results.push(self.benchmark_regex_matching().await);
        results.push(self.benchmark_syntax_highlighting().await);
        results.push(self.benchmark_tree_sitter_parsing().await);
        results.push(self.benchmark_string_generation().await);
        results.push(self.benchmark_workflow_listing().await);
        results.push(self.benchmark_string_offset_calculations().await);
        results.push(self.benchmark_sum_tree_operations().await);
        results.push(self.benchmark_shell_command_execution().await);
        results.push(self.benchmark_fuzzy_matching().await);

        BenchmarkSuite { results }
    }

    async fn benchmark_file_io(&self) -> BenchmarkResult {
        let name = "File I/O (Write/Read 1MB)".to_string();
        let iterations = 100;
        let file_size_mb = 1;
        let data = vec![0u8; file_size_mb * 1024 * 1024]; // 1MB of data
        let test_file = "benchmark_test_file.tmp";

        let start = Instant::now();
        let mut success = true;
        let mut details = String::new();

        for i in 0..iterations {
            match fs::write(test_file, &data).await {
                Ok(_) => {},
                Err(e) => { success = false; details = format!("Write error: {}", e); break; }
            }
            match fs::read(test_file).await {
                Ok(read_data) => {
                    if read_data.len() != data.len() {
                        success = false; details = "Read data size mismatch.".to_string(); break;
                    }
                },
                Err(e) => { success = false; details = format!("Read error: {}", e); break; }
            }
        }

        if fs::remove_file(test_file).await.is_err() {
            log::warn!("Failed to clean up benchmark test file: {}", test_file);
        }

        BenchmarkResult {
            name,
            duration: start.elapsed(),
            iterations,
        }
    }

    async fn benchmark_regex_matching(&self) -> BenchmarkResult {
        let name = "Regex Matching".to_string();
        let iterations = 1000;
        let text = "The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog.";
        let pattern = r"quick brown fox";
        let regex = Regex::new(pattern).unwrap();

        let start = Instant::now();
        let mut success = true;
        let mut matches_found = 0;

        for _ in 0..iterations {
            if let Some(_) = regex.find(text) {
                matches_found += 1;
            } else {
                success = false;
                break;
            }
        }

        BenchmarkResult {
            name,
            duration: start.elapsed(),
            iterations,
        }
    }

    async fn benchmark_syntax_highlighting(&self) -> BenchmarkResult {
        let name = "Syntax Highlighting (Rust)".to_string();
        let iterations = 50;
        let code = include_str!("../../src/main.rs"); // Use a real Rust file
        let syntax = self.syntax_set.find_syntax_by_extension("rs").unwrap();
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = Highlighter::new(theme);

        let start = Instant::now();
        let mut success = true;

        for _ in 0..iterations {
            for line in LinesWith::new(code, '\n') {
                let _ = highlighter.highlight_line(line, syntax, &self.syntax_set);
            }
        }

        BenchmarkResult {
            name,
            duration: start.elapsed(),
            iterations,
        }
    }

    async fn benchmark_tree_sitter_parsing(&self) -> BenchmarkResult {
        let name = "Tree-sitter Parsing (Bash)".to_string();
        let iterations = 100;
        let code = r#"
            #!/bin/bash
            function greet() {
                echo "Hello, $1!"
            }
            greet "World"
            for i in {1..10}; do
                echo "Count: $i"
            done
        "#;
        let mut parser = Parser::new();
        let language = tree_sitter_bash::language();
        parser.set_language(language).unwrap();

        let start = Instant::now();
        let mut success = true;

        for _ in 0..iterations {
            if parser.parse(code, None).is_none() {
                success = false;
                break;
            }
        }

        BenchmarkResult {
            name,
            duration: start.elapsed(),
            iterations,
        }
    }

    async fn benchmark_string_generation(&self) -> BenchmarkResult {
        let name = "String Generation".to_string();
        let iterations = 1000;
        let mut total_duration = Duration::new(0, 0);

        for _ in 0..iterations {
            let start = Instant::now();
            let _s: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 256);
            total_duration += start.elapsed();
        }

        BenchmarkResult {
            name,
            duration: total_duration / iterations,
            iterations,
        }
    }

    async fn benchmark_workflow_listing(&self) -> BenchmarkResult {
        let name = "Workflow Listing".to_string();
        let iterations = 100;
        let mut total_duration = Duration::new(0, 0);

        for _ in 0..iterations {
            let start = Instant::now();
            let _workflows = self.workflow_manager.list_workflows().await;
            total_duration += start.elapsed();
        }

        BenchmarkResult {
            name,
            duration: total_duration / iterations,
            iterations,
        }
    }

    async fn benchmark_string_offset_calculations(&self) -> BenchmarkResult {
        let name = "String Offset Calculations".to_string();
        let iterations = 1000;
        let long_string = "Hello, world! This is a very long string with many Unicode characters like 🚀✨🎉. It will be used to test string offset conversions between byte, char, and grapheme cluster indices. We need to ensure these conversions are fast and accurate for large text buffers. こんにちは世界！";

        let start = Instant::now();
        let mut success = true;

        for _ in 0..iterations {
            let byte_len = long_string.len();
            let char_len = long_string.chars().count();
            let grapheme_len = UnicodeSegmentation::graphemes(long_string, true).count();

            // Test conversions
            for i in 0..char_len {
                let byte_idx = self.string_offset_manager.char_to_byte_idx(long_string, i);
                if byte_idx.is_none() { success = false; break; }
                let char_idx = self.string_offset_manager.byte_to_char_idx(long_string, byte_idx.unwrap());
                if char_idx.is_none() || char_idx.unwrap() != i { success = false; break; }
            }
            if !success { break; }
        }

        BenchmarkResult {
            name,
            duration: start.elapsed(),
            iterations,
        }
    }

    async fn benchmark_sum_tree_operations(&self) -> BenchmarkResult {
        let name = "Sum Tree Operations".to_string();
        let iterations = 100;
        let num_elements = 10000;
        let mut values: Vec<f64> = (0..num_elements).map(|i| i as f64).collect();

        let start = Instant::now();
        let mut success = true;

        for _ in 0..iterations {
            let mut tree = self.sum_tree_manager.create_tree(values.clone());
            // Test update
            tree.update(num_elements / 2, 999.0);
            // Test query
            let _ = tree.query_prefix_sum(num_elements / 4);
            let _ = tree.query_index_by_sum(values.iter().sum::<f64>() / 2.0);
        }

        BenchmarkResult {
            name,
            duration: start.elapsed(),
            iterations,
        }
    }

    async fn benchmark_shell_command_execution(&self) -> BenchmarkResult {
        let name = "Shell Command Execution (echo)".to_string();
        let iterations = 50;
        let command = if cfg!(windows) { "cmd" } else { "bash" };
        let args = if cfg!(windows) { vec!["/C".to_string(), "echo Hello World!".to_string()] } else { vec!["-c".to_string(), "echo Hello World!".to_string()] };

        let start = Instant::now();
        let mut success = true;

        for _ in 0..iterations {
            let output = Command::new(command)
                .args(&args)
                .output()
                .await;

            match output {
                Ok(out) => {
                    if !out.status.success() {
                        success = false;
                        break;
                    }
                },
                Err(e) => {
                    success = false;
                    log::error!("Shell command execution error: {:?}", e);
                    break;
                }
            }
        }

        BenchmarkResult {
            name,
            duration: start.elapsed(),
            iterations,
        }
    }

    async fn benchmark_fuzzy_matching(&self) -> BenchmarkResult {
        let name = "Fuzzy Matching".to_string();
        let iterations = 1000;
        let mut total_duration = Duration::new(0, 0);

        let candidates = vec!["apple".to_string(), "banana".to_string(), "orange".to_string()];
        let query = "app".to_string();

        for _ in 0..iterations {
            let start = Instant::now();
            // Mock fuzzy matching
            let _results = self.fuzzy_match_manager.fuzzy_match(&query, &candidates);
            total_duration += start.elapsed();
        }

        BenchmarkResult {
            name,
            duration: total_duration / iterations,
            iterations,
        }
    }
}

pub fn init() {
    log::info!("performance/benchmarks module loaded");
}
