//!
//! ## Restictons:
//!
//! 1. *Don't leak threads*: When the main thread finishes, all other threads
//!    shut down. If you've accessed verona-rt resources in other threads,
//!    you'll have a bad time.

#![no_std]

pub mod cown;
pub mod log;
pub mod scheduler;
pub mod when;
