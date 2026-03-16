//! Execution-time behavior.

pub mod env;
pub mod fds;
pub mod path;
pub mod privilege;
pub mod run;
pub mod shell;
pub mod spawn;

pub use path::*;
pub use run::*;
