//! Config syntax and validation.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod validate;

pub use ast::*;
pub use validate::*;
