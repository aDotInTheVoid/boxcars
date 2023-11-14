use std::mem;

use verona_rt_sys as ffi;

/// Wrapper over `verona::cpp::Cown` and friends
///
/// ## Implementation Notes
///
/// In C++, all these classes have move and copy constructors, so care must be taken.
///
/// You can't return them by value over an FFI boundry, as the ABI will be wrong, and you'll
/// end up

pub struct CownPtr(pub(crate) ffi::CownPtr);

impl std::ops::Drop for CownPtr {
    fn drop(&mut self) {
        unsafe { ffi::cown_int_delete(&mut self.0) };
    }
}

impl Clone for crate::cown::CownPtr {
    fn clone(&self) -> crate::cown::CownPtr {
        unsafe {
            let mut new = mem::zeroed();
            ffi::cown_int_clone(&self.0, &mut new);
            Self(new.assume_init())
        }
    }
}

impl CownPtr {
    /// Must be inside a runtime.
    // TODO: Enforce that.
    pub fn new(value: i32) -> Self {
        unsafe {
            // The C++ code called here will read from the old value of cown to attempt to free it.
            // Luckely for us, `nullptr` is a valid value for a cown_ptr, and we can create one easily.
            let mut cown = mem::zeroed();
            ffi::cown_int_new(value, &mut cown);
            Self(cown.assume_init())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::scheduler::{with, with_leak_detector};

    #[test]
    fn new() {
        with_leak_detector(|| {
            let v = CownPtr::new(10);
            let v2 = v.clone();
            assert_eq!(v.0.lead_address(), v2.0.lead_address());
            drop(v);
            // TODO: Refcount check.
            drop(v2);
        })
    }

    #[test]
    fn new_minimal() {
        with(|| {
            CownPtr::new(10);
        })
    }

    #[test]
    fn clone_minimal() {
        with(|| {
            let v1 = CownPtr::new(42);
            _ = v1.clone();
        })
    }

    #[test]
    fn lead_detector_works() {
        // TODO: This should panic.
        with_leak_detector(|| {
            let v = CownPtr::new(666);
            mem::forget(v);
        });
    }
}
