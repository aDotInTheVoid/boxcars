//! Low level FFI bindings to the [verona runtime](https://github.com/microsoft/verona-rt)

// #[repr(C)]
// struct CownPtr(*const ());

use std::mem;

#[repr(C)]
#[derive(Clone, Copy)]
/// A reference to a `verona::rt::Scheduler`.
///
/// The underlying Schedular is a singleton, but multiple `Schedular`
/// structs can exist, which will all point to the same singleton.
///
/// Create with [`scheduler_get`]
pub struct Scheduler(*const ());

pub type UseInt = extern "C" fn(&mut AquiredCown, *const ());

#[repr(C)]
/// This is a reference cointed pointer, so embeders shouldn't
/// implement Copy.
///
/// Must not be moved directly over the FFI boundry, as C++ and rust
/// use different calling conventions.
pub struct CownPtr(*const ());

impl CownPtr {
    pub fn lead_address(&self) -> *const () {
        self.0
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AquiredCown(*const ());

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

    /// Extreamly racy.
    pub fn schedular_set_detect_leaks(detect_leaks: bool);
    pub fn schedular_has_leaks() -> bool;

    pub fn cown_int_new(value: i32, cown: &mut mem::MaybeUninit<CownPtr>);
    pub fn cown_int_delete(cown: &mut CownPtr);
    pub fn cown_int_clone(input: &CownPtr, output: &mut mem::MaybeUninit<CownPtr>);

    pub fn cown_int_when1(cown: &CownPtr, func: UseInt, data: *const ());
    pub fn cown_get_ref(cown: &AquiredCown) -> *mut i32;
    pub fn cown_get_cown(cown: &AquiredCown, out: &mut mem::MaybeUninit<CownPtr>);

    pub fn enable_logging();
    pub fn boxcar_log(ptr: *const u8, len: usize);
}

#[test]
fn add_ints() {
    unsafe {
        assert_eq!(boxcars_add(1, 2), 3);
    }
}
