//! Low level FFI bindings to the [verona runtime](https://github.com/microsoft/verona-rt)

// #[repr(C)]
// struct CownPtr(*const ());

#[repr(C)]
#[derive(Clone, Copy)]
/// A reference to a `verona::rt::Scheduler`.
///
/// The underlying Schedular is a singleton, but multiple `Schedular`
/// structs can exist, which will all point to the same singleton.
///
/// Create with [`scheduler_get`]
pub struct Scheduler(*const ());

#[link(name = "boxcar_bindings")]
extern "C" {
    #[cfg(test)]
    /// Calculates a+b. Used for testing purposed only.
    fn boxcars_add(a: i32, b: i32) -> i32;

    /// Returns the global `Scheduler`.
    ///
    /// ## Safety
    ///
    /// Safe
    pub fn scheduler_get() -> Scheduler;

    /// Initialize the schedular with the given number of threads.
    ///
    /// ## Safety
    ///
    /// - `n_threads` must not be zero.
    /// - The scheduler must not be "initialized"
    ///   - TODO: What exactly does this mean??
    pub fn scheduler_init(schedular: Scheduler, n_threads: usize);

    /// Run the schedular.
    ///
    /// This will "de-initialize" the scheduler, allowing you to "re-initialize"
    /// it.
    ///
    /// ## Safety
    ///
    /// - Unclear. It's probably a bad idea to call it multiple times without
    ///   re-initializing the schedular.
    pub fn scheduler_run(schedular: Scheduler);
}

#[test]
fn add_ints() {
    unsafe {
        assert_eq!(boxcars_add(1, 2), 3);
    }
}
