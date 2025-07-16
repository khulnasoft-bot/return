pub mod command_palette;
pub mod ai_sidebar;
pub mod collapsible_block;
pub mod ratatui_block; // This module is kept for completeness but not used in the Iced GUI.

use log::info;

pub fn init() {
    info!("UI module initialized.");
}
