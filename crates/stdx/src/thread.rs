//! Alternative implementation of [`std::thread::scope`] and friends that trys
//! to run thread-local destructors.
//!
//! It solves [rust#116237](https://github.com/rust-lang/rust/issues/116237) by
//! storing a list of [`std::thread::ScopedJoinHandle`] to join on drop.
//!
//! See [boxcars#21](https://github.com/aDotInTheVoid/boxcars/issues/21) for
//! motivation, and thanks to Mara Bos for pointing out that
//! [`std::thread::scope`] won't wait for thread-local destructors.

use std::{mem, ops, sync::Mutex, thread as imp};

pub struct Scope<'scope, 'env: 'scope> {
    inner: &'scope imp::Scope<'scope, 'env>,
    handles: Mutex<Vec<imp::ScopedJoinHandle<'scope, ()>>>,
}

pub fn scope<'env, F, T>(f: F) -> T
where
    F: for<'scope> FnOnce(Scope<'scope, 'env>) -> T,
{
    std::thread::scope(|inner| {
        let scope = Scope {
            inner,
            handles: Mutex::default(),
        };
        f(scope)
    })
}

impl<'scope, 'env> Scope<'scope, 'env> {
    // We don't have a generic param because we need a list of homogenous list of join handles
    // This could probably be solved with a trait object if it's needed.
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        let handle = self.inner.spawn(f);
        self.handles.lock().unwrap().push(handle);
    }
}

impl ops::Drop for Scope<'_, '_> {
    fn drop(&mut self) {
        if imp::panicking() {
            return;
        }

        let handles = mem::take(self.handles.get_mut().unwrap());
        for h in handles {
            h.join().unwrap();
        }
    }
}
