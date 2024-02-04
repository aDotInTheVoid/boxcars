//! ```rust
//! # use verona_rt::{
//! #     cown::CownPtr,
//! #     scheduler::with,
//! #     when::{when, when2},
//! # };
//!
//! # with(|| {
//! let string = CownPtr::new(String::new());
//! let vec = CownPtr::new(Vec::new());
//!
//! when(&string, |mut s| {
//!     assert_eq!(&*s, "");
//!     s.push_str("foo");
//! });
//! when(&vec, |mut v| {
//!     assert_eq!(&*v, &[]);
//!     v.push(101);
//! });
//! when2(&string, &vec, |mut s, mut v| {
//!     assert_eq!(&*s, "foo");
//!     assert_eq!(&*v, &[101]);
//!     s.push_str("bar");
//!     v.push(666);
//! });
//!
//! when(&string, |s| assert_eq!(&*s, "foobar"));
//! when(&vec, |v| assert_eq!(&*v, &[101, 666]));
//! # });
//! ```
//!
//! ## Current Status
//!
//! This is a research project, and is at an early stage of development. It is not
//! ready for use outside of research.
//!
//! ## Restrictions:
//!
//! 1. *Don't leak threads*: When the main thread finishes, all other threads
//!    shut down. If you've accessed verona-rt resources in other threads,
//!    you'll have a bad time.
//!
//!
//! ## Current Status
//!
//! This is a research project, and is at an early stage of development. It is not
//! ready for use outside of research.

// It'd be nice, see #17, but we need mutex's
// #![no_std]

pub mod cown;
pub mod log;
pub mod scheduler;
pub mod when;
