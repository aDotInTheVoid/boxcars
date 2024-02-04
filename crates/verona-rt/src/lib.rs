#![doc = include_str!("../../../readme.md")]
//!
//! ## Restictons:
//!
//! 1. *Don't leak threads*: When the main thread finishes, all other threads
//!    shut down. If you've accessed verona-rt resources in other threads,
//!    you'll have a bad time.

// It'd be nice, see #17, but we need mutex's
// #![no_std]

pub mod cown;
pub mod log;
pub mod scheduler;
pub mod when;
