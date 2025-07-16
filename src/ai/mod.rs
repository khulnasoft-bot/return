//! This module provides the core AI functionalities for NeoTerm,
//! including the AI assistant, context management, and integration
//! with various AI providers.
//!
//! It serves as the entry point for all AI-related operations within the application.

use log::info;

pub mod assistant;
pub mod context;
pub mod providers;
pub mod prompts;

/// Initializes the AI module.
pub fn init() {
    info!("AI module initialized.");
}
