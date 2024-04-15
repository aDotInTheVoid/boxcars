use core::{fmt, marker::PhantomData, mem::MaybeUninit, ptr};

use verona_rt_sys as ffi;

use crate::descriptor::get_desc;

// See docs/layout.md for how this works.

pub struct CownPtr<T> {
    pub(crate) cown_ptr: ffi::CownPtr,
    // TODO: Is this right wrt send/sync.
    _marker: PhantomData<T>,
}

#[repr(C)]
#[derive(Debug)]
// Corresponds to verona::rt::Cown.
struct OpaqueCown {
    _marker: MaybeUninit<[*const (); 4]>,
}

#[repr(C)]
pub(crate) struct CownData<T> {
    // Must be first, so we can convert pointers between the two.
    cown: OpaqueCown,
    data: T,
}

pub(crate) fn cown_to_data<T>(ptr: *mut ()) -> *mut T {
    debug_assert!(!ptr.is_null());
    debug_assert!((ptr as usize) & 15 == 0, "{ptr:p} not 16 bit aligned");

    let p = ptr as *mut CownData<T>;

    unsafe { ptr::addr_of_mut!((*p).data) }
}

impl<T> CownPtr<T> {
    fn data_ptr(&self) -> *mut T {
        cown_to_data(self.cown_ptr.addr())
    }

    #[cfg(test)]
    unsafe fn yolo_data(&mut self) -> &mut T {
        &mut *(self.data_ptr() as *mut T)
    }
}

impl<T> fmt::Pointer for CownPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.cown_ptr.addr(), f)
    }
}

impl<T> core::ops::Drop for CownPtr<T> {
    fn drop(&mut self) {
        unsafe { ffi::boxcars_release_object(self.cown_ptr) };
    }
}

impl<T> Clone for crate::cown::CownPtr<T> {
    fn clone(&self) -> Self {
        unsafe {
            ffi::boxcars_acquire_object(self.cown_ptr);
            Self {
                cown_ptr: self.cown_ptr,
                _marker: PhantomData,
            }
        }
    }
}

impl<T> CownPtr<T> {
    /// Must be inside a runtime.
    // TODO: Enforce that.
    pub fn new(value: T) -> Self {
        unsafe {
            let desc = get_desc::<T>();
            let cown_ptr = ffi::boxcars_allocate_cown(&desc);

            let this = Self {
                cown_ptr,
                _marker: PhantomData,
            };
            ptr::write(this.data_ptr(), value);

            this
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    use crate::scheduler::{self, with, with_leak_detector};

    #[test]
    fn new() {
        with_leak_detector(|| {
            let v = CownPtr::new(10);
            let v2 = v.clone();
            assert_eq!(v.cown_ptr.addr(), v2.cown_ptr.addr());
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
            assert_ne!(v2.cown_ptr.addr(), ptr::null_mut());
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
    fn read_modify_write() {
        scheduler::with_leak_detector(|| {
            let mut c = CownPtr::new([0; 100]);
            assert_ne!(c.cown_ptr.addr(), ptr::null_mut());
            {
                let c = unsafe { c.yolo_data() };
                for (n, el) in c.iter_mut().enumerate() {
                    assert_eq!(*el, 0);
                    *el = n;
                }
            }

            let mut c1 = c.clone();
            assert_ne!(c1.cown_ptr.addr(), ptr::null_mut());

            {
                for (n, el) in unsafe { c1.yolo_data() }.iter_mut().enumerate() {
                    assert_eq!(*el, n);
                    *el *= 2;
                }
            }

            let mut c2 = c.clone();
            assert_ne!(c2.cown_ptr.addr(), ptr::null_mut());
            {
                for (n, el) in unsafe { c2.yolo_data() }.iter().enumerate() {
                    assert_eq!(*el, n * 2);
                }
            }
        })
    }

    #[test]
    fn write_stress() {
        fn stress_once<const N: usize>() {
            let mut x = CownPtr::new([0u8; N]);

            unsafe {
                for i in x.yolo_data() {
                    assert_eq!(*i, 0);
                    *i = 0xCC;
                }
            }

            let mut x2 = x.clone();
            unsafe {
                for i in x2.yolo_data() {
                    assert_eq!(*i, 0xCC);
                    *i = 0x33;
                }
            }
            drop(x2);

            unsafe {
                for i in x.yolo_data() {
                    assert_eq!(*i, 0x33);
                }
            }

            drop(x);
        }

        fn repeat_alloc<const N: usize>() {
            for _ in 0..100 {
                stress_once::<N>();
            }
        }

        scheduler::with_leak_detector(|| {
            repeat_alloc::<3932>();
            repeat_alloc::<3719>();
            repeat_alloc::<1477>();
            repeat_alloc::<414>();
            repeat_alloc::<163>();
            repeat_alloc::<4>();
            repeat_alloc::<3>();
            repeat_alloc::<2>();
            repeat_alloc::<1>();
            repeat_alloc::<0>();
        });
    }

    struct WriteOnDrop<'a>(&'a Cell<bool>);
    impl Drop for WriteOnDrop<'_> {
        fn drop(&mut self) {
            assert_eq!(self.0.get(), false);
            self.0.set(true);
        }
    }

    #[test]
    fn dtor() {
        scheduler::with(|| {
            let flag = Cell::new(false);
            let cown = CownPtr::new(WriteOnDrop(&flag));

            assert_eq!(flag.get(), false);
            drop(cown);
            assert_eq!(flag.get(), true);
        })
    }

    #[test]
    fn dtor_clone() {
        scheduler::with(|| {
            let flag = Cell::new(false);
            let cown = CownPtr::new(WriteOnDrop(&flag));

            assert_eq!(flag.get(), false);
            let cown2 = cown.clone();
            assert_eq!(flag.get(), false);
            drop(cown);
            assert_eq!(flag.get(), false);
            drop(cown2);
            assert_eq!(flag.get(), true);
        })
    }
}
