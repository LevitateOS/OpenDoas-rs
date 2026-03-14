/*
OpenDoas-rs - Privilege escalation utility
Copyright (C) 2023  TheDcoder <TheDcoder@protonmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

pub mod app;
pub mod auth;
pub mod cli;
pub mod config;
pub mod exec;
pub mod logging;
pub mod persist;
pub mod platform;
pub mod policy;

pub mod command {
    pub use crate::cli::args::{Command, Execute};
    pub use crate::cli::usage::{print_error, print_error_and_exit};
}

pub use cli::mode::Mode;
pub use policy::rule::{EnvDirective, Rule, RuleAction, RuleIdentity, RuleOpts, Rules};
