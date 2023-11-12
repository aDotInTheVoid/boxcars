use std::sync::Mutex;

use ffi::scheduler_get;
/// Access to the verona schedular.
///
/// ## Global singleton
///
/// In each program there is exactly one global schedular. This is a big pile
/// of global state, that must be carefully managed.
///
/// TODO: Understand init/run well.
use verona_rt_sys as ffi;

fn get() -> ffi::Scheduler {
    // TODO: Is it worth caching the returned pointer?

    // SAFETY: scheduler_get is safe to call.
    unsafe { ffi::scheduler_get() }
}

static SCHED_LOCK: Mutex<()> = Mutex::new(());

pub fn with<T>(f: impl FnOnce() -> T) -> T {
    // Note: we don't care if it was poisoned.
    let _lock = SCHED_LOCK.lock();

    unsafe {
        ffi::scheduler_init(scheduler_get(), 8);
    }

    struct DropGuard;
    impl Drop for DropGuard {
        fn drop(&mut self) {
            unsafe { ffi::scheduler_run(get()) }
        }
    }

    // Use a drop guard to clean up scheduler resources even in the case that
    // The closure panics.
    let dg = DropGuard;
    let result = f();
    drop(dg);

    result
}

/// Initialize the schedular with the given number of threads.
///
/// ## Safety
///
/// The global
// pub unsafe fn init(n_threads: NonZeroUsize) {
//     ffi::scheduler_init(get(), n_threads.get())
// }

// pub unsafe fn run() {
//     ffi::scheduler_run(get())
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_run() {
        with(|| {});
    }

    #[test]
    fn run_from_multi_threads() {
        std::thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    for _ in 0..100 {
                        with(|| {});
                    }
                });
            }
        });
    }

    #[test]
    fn panic_safe() {
        std::thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    for _ in 0..10 {
                        let r = std::panic::catch_unwind(|| with(|| panic!("lol lmao")));
                        let r_err = r.unwrap_err();
                        let s = r_err.downcast::<&str>().unwrap();
                        assert_eq!(&**s, "lol lmao");
                    }
                });
            }
        })
    }
}
