//! # Behavior Oriented Concurrency in Rust
//!
//! For an introduction to the Behavior Oriented Concurrency model, see the
//! [OOPSLA 2023 Paper](https://dl.acm.org/doi/10.1145/3622852). This library
//! aims to provide idiomatic Rust bindings to the [Verona runtime](https://github.com/microsoft/verona-rt),
//! as introduced in that paper.
//!
//! ```rust
//! # use verona_rt::*;
//! # with_scheduler(|| {
//! let string = Cown::new(String::new());
//! let vec = Cown::new(Vec::<i32>::new());
//!
//! when(&string, |mut s| {
//!     assert_eq!(&*s, "");
//!     s.push_str("foo");
//! });
//! when(&vec, |mut v| {
//!     assert_eq!(&*v, &[]);
//!     v.push(101);
//! });
//! when((&string, &vec), |(mut s, mut v)| {
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
//! This is a research project. It is not ready for use outside of research.
//!
//! ## Restrictions:
//!
//! Note: This list in non-exhaustive. If you do anything weird, you may well
//! get a crash deep inside the gut of verona-rt. That's not to say you
//! shouldn't, just a warning about how robust this is at the moment. In fact,
//! if you do discover something not listed here, please [let me
//! know](https://github.com/aDotInTheVoid/boxcars/issues/new).
//!
//! 1. *Don't leak threads*: When the main thread finishes, all other threads
//!    shut down. If you've accessed verona-rt resources in other threads,
//!    you'll have a bad time.
//! 2. *Run everything inside a schedular*: Use [`with_scheduler`] to set up and
//!    tear down the global schedular state.
//! 3. *Don't panic*: If you panic with the schedular, arbitrarily bad things
//!    happen.
// !    I'm working on solving this, but it's on the backburner for now.
//! 4. *Don't make a load of schedulers*: Everything should run with the same schedular.
//!    If you call [`with_scheduler``] on a load of thread, your going to have a bad day
//!    (unless you like debugging non-reproducible segfaults :)).
//! 5. *Run thread local destructors*: (Especially if using the leak-dececor), if you don't
//!     ensure that `thread_local` destructors are run, you'll end up with racy false-positives.
//!
//!     Notably [`std::thread::scope`], doesn't guarantee to run them [if you don't call `.join()`
//!     on the handles](https://github.com/rust-lang/rust/issues/116237). Thanks to Mara Bos for
//!     pointing this out to me.

// It'd be nice, see #17, but we need mutex's
// #![no_std]

mod cown;
mod descriptor;
mod lambdas;
mod log;
mod scheduler;
mod variadic_when;
mod when;

#[cfg(test)]
mod tests;

pub use cown::Cown;
pub use log::{log, log_snmalloc};
pub use scheduler::{with_leak_detector, with_n_threads, with_scheduler};
pub use variadic_when::{when, CownCollection};
pub use when::AcquiredCown;
pub use when::{when1, when2, when3, when4, when5, when6, when7, when8, when9};

pub use lambdas::when0;

pub(crate) use lambdas::schedule_lambda;
