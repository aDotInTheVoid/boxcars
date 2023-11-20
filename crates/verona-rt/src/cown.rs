use std::{marker::PhantomData, mem, ptr};

use verona_rt_sys as ffi;

/// Wrapper over `verona::cpp::Cown` and friends
///
/// ## Implementation Notes
///
/// In C++, all these classes have move and copy constructors, so care must be taken.
///
/// You can't return them by value over an FFI boundry, as the ABI will be wrong, and you'll
/// end up

// +-------+-----------+----------------------+
// | Cown  | DtorThunk | Rust Manged Whatever |
// +-------+-----------+----------------------+
// | ActualCown        |
// +-------------------+----------------------+
// | CownData                                 |
// +------------------------------------------+

pub struct CownPtr<T> {
    ptr: ffi::CownPtr,
    // TODO: Is this right wrt send/sync.
    _marker: PhantomData<CownData<T>>,
}

impl<T> CownPtr<T> {
    fn cown_data(&self) -> *mut CownData<T> {
        self.ptr.lead_address() as _
    }

    /// Safety: lol
    #[cfg(test)]
    unsafe fn yolo_data(&mut self) -> &mut T {
        &mut (*self.cown_data()).data
    }
}

#[repr(C)]
struct ActualCown {
    _marker: std::mem::MaybeUninit<[u64; 4]>,
}

#[repr(C)]
struct CownData<T> {
    // Must be first, so we can convert pointers between the two.
    cown: ActualCown,
    data: T,
}

impl<T> std::ops::Drop for CownPtr<T> {
    fn drop(&mut self) {
        unsafe { ffi::boxcar_cownptr_drop(&mut self.ptr) };
    }
}

impl<T> Clone for crate::cown::CownPtr<T> {
    fn clone(&self) -> Self {
        unsafe {
            let mut new = mem::zeroed();
            ffi::boxcar_cownptr_clone(&self.ptr, &mut new);
            Self {
                ptr: new,
                _marker: PhantomData,
            }
        }
    }
}

extern "C" fn dummy_drop(ptr: *const ()) {
    dbg!(ptr);
}

impl<T> CownPtr<T> {
    /// Must be inside a runtime.
    // TODO: Enforce that.
    pub fn new(value: T) -> Self {
        unsafe {
            // The C++ code called here will read from the old value of cown to attempt to free it.
            // Luckely for us, `nullptr` is a valid value for a cown_ptr, and we can create one easily.
            let mut cown_ptr = mem::zeroed();

            ffi::boxcar_cownptr_new(
                // TODO: Why is this needed? AAAH. +8 doesn't work.
                std::mem::size_of::<CownData<T>>() + 9,
                dummy_drop,
                &mut cown_ptr,
            );

            let this = Self {
                ptr: cown_ptr,
                _marker: PhantomData,
            };

            let data_ptr = ptr::addr_of_mut!((*this.cown_data()).data);

            data_ptr.write(value);

            this
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::scheduler::{self, with, with_leak_detector};

    #[test]
    fn new() {
        with_leak_detector(|| {
            let v = CownPtr::new(10);
            let v2 = v.clone();
            assert_eq!(v.ptr.lead_address(), v2.ptr.lead_address());
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
    fn clone_notnull() {
        with(|| {
            let v1 = CownPtr::new(10);
            let v2 = v1.clone();
            assert_ne!(v2.ptr.lead_address(), ptr::null());
        })
    }

    #[test]
    fn leak_detector_new() {
        unsafe {
            ffi::enable_logging();
        }

        with_leak_detector(|| {
            let x = CownPtr::new(1010);
            let y = x.clone();
            drop(x);
            drop(y);
        });
    }

    #[test]
    fn actualcown_constats_right() {
        let mut size = 0;
        let mut align = 0;

        unsafe {
            ffi::boxcar_actualcown_info(&mut size, &mut align);
        }

        // TODO: Get these values over FFI.
        assert_eq!(std::mem::size_of::<ActualCown>(), size);
        assert_eq!(std::mem::align_of::<ActualCown>(), align);
    }

    #[test]
    fn read_modify_write() {
        scheduler::with_leak_detector(|| {
            let mut c = CownPtr::new([0; 100]);
            assert_ne!(c.ptr.lead_address(), ptr::null());
            {
                let c = unsafe { c.yolo_data() };
                for (n, el) in c.iter_mut().enumerate() {
                    assert_eq!(*el, 0);
                    *el = n;
                }
            }

            let mut c1 = c.clone();
            assert_ne!(c1.ptr.lead_address(), ptr::null());

            {
                for (n, el) in unsafe { c1.yolo_data() }.iter_mut().enumerate() {
                    assert_eq!(*el, n);
                    *el *= 2;
                }
            }

            let mut c2 = c.clone();
            assert_ne!(c2.ptr.lead_address(), ptr::null());
            {
                for (n, el) in unsafe { c2.yolo_data() }.iter().enumerate() {
                    assert_eq!(*el, n * 2);
                }
            }
        })
    }
}
