//! High-level application flows.

pub mod check;
pub mod execute;

pub use check::render_check_result;
pub use execute::{load_rules, ConfigRequest};
