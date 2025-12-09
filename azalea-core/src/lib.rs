//! # azalea-core
//!
//! Crate responsible for implementing:
//! - main application state
//! - command line arguments
//! - configuration structs
//! - client-server through unix sockets

pub mod app;
pub mod cli;
pub mod config;
pub mod dbus;
pub mod error;
use azalea_log as log;
pub mod monitor;
pub mod socket;
