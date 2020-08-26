mod models;
mod gui;
pub mod manual_migrations;
pub mod files_discovery;
pub mod actors;
pub mod msgs;

pub use gui::start_gui;

#[derive(rust_embed::RustEmbed)]
#[folder = "assets"]
struct Asset;
