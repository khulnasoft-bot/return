use log::info;

pub mod prompts;
pub mod assistant;
pub mod context;
pub mod providers;

pub fn init() {
    info!("ai module loaded");
}
