//! ```rust
//! # use verona_rt::{
//! #     cown::CownPtr,
//! #     scheduler::with,
//! #     when::{when, when2},
//! # };
//! #
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
//! Note: This list in non-exhaustive. If you do anything weird, you may well
//! get a crash deep inside the gut of verona-rt. That's not to say you shouldn't,
//! just a warning about how robust this is at the moment. In fact, if you do
//! discover something not listed here, please [let me know](https://github.com/aDotInTheVoid/boxcars/issues/new).
//!
//! 1. *Don't leak threads*: When the main thread finishes, all other threads
//!    shut down. If you've accessed verona-rt resources in other threads,
//!    you'll have a bad time.
//! 2. *Run everything inside a schedular*: Use [`scheduler::with`] to set up and
//!    tear down the global schedular state.
//! 3. *Don't panic*: If you panic with the schedular, arbitrarily bad things happen.
// !    I'm working on solving this, but it's on the backburner for now.
//! 4. *Don't make a load of schedulers*: Everything should run with the same schedular.
//!    If you call [`scheduler::with``] on a load of thread, your going to have a bad day
//!    (unless you like debugging non-reproducible segfaults :)).

// It'd be nice, see #17, but we need mutex's
// #![no_std]

mod cown;
mod log;
mod scheduler;
mod when;

pub use cown::CownPtr;
pub use scheduler::with as with_scheduler;
pub use when::{when, when2, AcquiredCown};
