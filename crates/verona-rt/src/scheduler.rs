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

struct DropGuard;
impl Drop for DropGuard {
    fn drop(&mut self) {
        unsafe { ffi::scheduler_run(get()) }
    }
}

pub fn with<T: Send>(f: impl FnOnce() -> T + Send) -> T {
    with_inner(f, true)
}

pub fn with_inner<T, F: FnOnce() -> T>(f: F, detect_leaks: bool) -> T {
    let lock = SCHED_LOCK.lock();

    unsafe {
        ffi::scheduler_init(scheduler_get(), 1);

        if detect_leaks {
            ffi::schedular_set_detect_leaks(true);
        }
    }

    // Use a drop guard to clean up scheduler resources even in the case that
    // The closure panics.
    let dg = DropGuard;
    let result = f();
    drop(dg); // Calls Scheduler.run

    if detect_leaks {
        unsafe {
            if ffi::schedular_has_leaks() {
                panic!("leaks detected");
            }
            ffi::schedular_set_detect_leaks(false)
        }
    }

    drop(lock);

    result
}

pub fn with_leak_detector<T>(f: impl FnOnce() -> T) -> T {
    with_inner(f, true)
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
    use crate::cown;

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

    // #[test]
    // fn concurrent_leak_detector() {
    //     fn do_a_clone() {
    //         _ = cown::CownPtr::new(101).clone();
    //     }

    //     for _ in 0..1000 {
    //         let t1 = std::thread::spawn(|| with(|| do_a_clone()));
    //         let t3 = std::thread::spawn(|| with_leak_detector(|| do_a_clone()));

    //         t1.join().unwrap();
    //         t3.join().unwrap();
    //     }
    // }
}
