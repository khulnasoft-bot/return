pub mod command_palette;
pub mod ai_sidebar;
pub mod collapsible_block;
pub mod ratatui_block; // This module is kept for completeness but not used in the Iced GUI.
pub mod terminal_command_display; // New module for terminal command display

use log::info;

pub fn init() {
    info!("UI module initialized.");
}
