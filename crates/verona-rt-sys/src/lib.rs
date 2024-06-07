//! Low level FFI bindings to the [verona runtime](https://github.com/microsoft/verona-rt)
//!
//! This is an implementation detail of the verona-rt crate.
//!
//! This is a research project, and is at an early stage of development. It is not
//! ready for use outside of research.
#![cfg_attr(not(test), no_std)]
use core::fmt;

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

/// Equivalent to `rt::Cown*` on the C++ side.
///
/// This is a reference cointed pointer, so embeders shouldn't
/// implement Copy.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct CownPtr {
    addr: *mut (),
}

impl fmt::Pointer for CownPtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.addr)
    }
}

impl CownPtr {
    pub fn addr(self) -> *mut () {
        self.addr
    }
}

#[repr(C)]
#[derive(Debug)]
/// `Corresponds to rt::Cown`.
///
/// Rust code shouldn't inspect the contents of it, but use it for size/
/// allignment/pointer arithmatic.
pub struct OpaqueCown {
    _marker: core::mem::MaybeUninit<[*const (); 3]>,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct WorkPtr(*mut ());

#[repr(C)]
pub struct Slot {
    pub cown: CownPtr,
    _scheginfo: core::sync::atomic::AtomicUsize,
}

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

    pub fn enable_logging();
    pub fn boxcars_dump_flight_recorder();

    pub fn boxcars_sched_lambda(
        n_cowns: usize,
        cowns: *const CownPtr,
        f: extern "C" fn(WorkPtr),
        payload_size: usize,
        payload: *const (),
    );
    pub fn boxcars_preinvoke(
        work: WorkPtr,
        slots: &mut *const Slot,
        body: &mut *const (),
        count: &mut usize,
    );
    pub fn boxcars_postinvoke(work: WorkPtr);

    pub fn boxcar_log_cstr(ptr: *const core::ffi::c_char);
    pub fn boxcar_log_usize(n: usize);
    pub fn boxcar_log_ptr(p: *const ());
    pub fn boxcar_log_endl();
    // TODO: Should this be c_char?
    pub fn boxcars_snmalloc_message(ptr: *const u8, len: usize);
}

#[test]
fn cown_size_and_align() {
    #[link(name = "boxcar_bindings")]
    extern "C" {
        fn boxcars_test_cown_info(size: &mut usize, align: &mut usize);
    }
    let mut size = 0;
    let mut align = 0;
    unsafe {
        boxcars_test_cown_info(&mut size, &mut align);
    }
    assert_eq!(size, std::mem::size_of::<OpaqueCown>());
    assert_eq!(align, std::mem::align_of::<OpaqueCown>())
}

#[test]
fn slot_size_and_align() {
    extern "C" {
        fn boxcars_test_slot_info(size: &mut usize, align: &mut usize);
    }

    let (mut size, mut align) = (0, 0);
    unsafe { boxcars_test_slot_info(&mut size, &mut align) };
    assert_eq!(size, std::mem::size_of::<Slot>());
    assert_eq!(align, std::mem::align_of::<Slot>());
}
