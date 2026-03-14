//! Platform-facing lookups.

pub mod groups;
pub mod passwd;
pub mod tty;

pub use groups::{current_group_ids, current_group_info, current_group_names};
pub use passwd::{current_passwd, parse_target_uid, target_passwd};
pub use tty::*;
