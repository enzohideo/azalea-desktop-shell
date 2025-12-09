//! # azalea
//!
//! This crate works as the main application and as a library that re-exports the inner crates

#[doc(inline)]
pub use azalea_core as core;

#[doc(inline)]
pub use azalea_log as log;

#[doc(inline)]
pub use azalea_shell as shell;

#[doc(inline)]
pub use azalea_service as service;
