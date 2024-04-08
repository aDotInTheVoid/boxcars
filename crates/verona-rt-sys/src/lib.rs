//! Low level FFI bindings to the [verona runtime](https://github.com/microsoft/verona-rt)
//!
//! This is an implementation detail of the verona-rt crate.
//!
//! This is a research project, and is at an early stage of development. It is not
//! ready for use outside of research.

use descriptor::Descriptor;

pub mod descriptor;
mod vsizeof;
pub use vsizeof::vsizeof;

#[repr(transparent)]
#[derive(Clone, Copy)]
/// A reference to a `verona::rt::Scheduler`.
///
/// The underlying Schedular is a singleton, but multiple `Schedular`
/// structs can exist, which will all point to the same singleton.
///
/// Create with [`scheduler_get`]
pub struct Scheduler(*mut ());

#[repr(transparent)]
/// This is a reference cointed pointer, so embeders shouldn't
/// implement Copy.
///
/// Equivalent to `rt::Cown*` on the C++ side.
#[derive(Clone, Copy)]
pub struct CownPtr(*mut ());

impl CownPtr {
    pub fn addr(&self) -> *mut () {
        self.0
    }
}
impl AcquiredCown {
    pub fn addr(&self) -> *mut () {
        self.0
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AcquiredCown(*mut ());

pub type Dtor = extern "C" fn(*mut ());

#[link(name = "boxcar_bindings")]
extern "C" {

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

    pub fn boxcars_acquire_object(cown: CownPtr);
    pub fn boxcars_release_object(cown: CownPtr);

    pub fn boxcars_allocate_cown(descriptor: &'static Descriptor) -> CownPtr;

    // pub fn boxcar_when1(
    //     cown: &CownPtr,
    //     func: extern "C" fn(&mut AcquiredCown, *mut ()),
    //     data: *mut (),
    // );
    // pub fn boxcar_when2(
    //     c1: &CownPtr,
    //     c2: &CownPtr,
    //     func: extern "C" fn(&mut AcquiredCown, &mut AcquiredCown, *mut ()),
    //     data: *mut (),
    // );

    pub fn enable_logging();
    pub fn dump_flight_recorder();

    pub fn boxcar_log_cstr(ptr: *const std::ffi::c_char);
    pub fn boxcar_log_usize(n: usize);
    pub fn boxcar_log_ptr(p: *const ());
    pub fn boxcar_log_endl();
}
