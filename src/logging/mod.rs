//! Audit and logging behavior.

pub mod audit;

pub use audit::{log_denied_command, log_failed_auth, log_permitted_command, log_tty_required};
