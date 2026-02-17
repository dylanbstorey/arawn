//! UI rendering components.

pub mod chat;
pub mod command_popup;
pub mod input;
mod layout;
pub mod logs;
pub mod palette;
pub mod sessions;
pub mod sidebar;

pub use command_popup::{CommandInfo, CommandPopup};
pub use layout::render;
