//! alf — library core.
//!
//! This crate holds the domain logic (catalog, skills, manifest, lock, config,
//! scaffold) and the command functions. It has no dependency on clap or stdout,
//! so it can be exercised directly from tests. The binary (`src/main.rs`) is the
//! thin CLI layer that parses arguments and prints results.

pub mod catalog;
pub mod commands;
pub mod config;
pub mod detect;
pub mod error;
pub mod fsops;
pub mod lock;
pub mod manifest;
pub mod project;
pub mod scaffold;
pub mod seeds;
pub mod skill;
pub mod version;

pub use error::AlfError;
