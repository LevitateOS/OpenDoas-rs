//! Authentication backends.

#[cfg(auth = "none")]
pub mod none;
#[cfg(auth = "pam")]
pub mod pam;
#[cfg(auth = "plain")]
pub mod plain;

#[cfg(auth = "none")]
pub use none::*;
#[cfg(auth = "pam")]
pub use pam::*;
#[cfg(auth = "plain")]
pub use plain::*;
