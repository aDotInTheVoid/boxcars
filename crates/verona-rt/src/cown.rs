use core::fmt;
use std::{
    fmt::Debug,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr,
};

use verona_rt_sys as ffi;

// See docs/layout.md for how this works.

pub struct CownPtr<T> {
    pub(crate) cown_ptr: ffi::CownPtr,
    // TODO: Is this right wrt send/sync.
    _marker: PhantomData<T>,
}

#[repr(C)]
/// It's never safe to dereference this type, or even to construct one.
pub(crate) struct CownDataToxic<T> {
    // Must be first, so we can convert pointers between the two.
    cown: ActualCown,
    data: T,
}

pub(crate) fn cown_to_data<T>(ptr: *mut ()) -> *mut T {
    debug_assert!(!ptr.is_null());
    debug_assert!((ptr as usize) & 15 == 0, "{ptr:p} not 16 bit aligned");

    let p = ptr as *mut CownDataToxic<T>;

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

#[repr(C)]
#[derive(Debug)]
struct ActualCown {
    _marker: MaybeUninit<[*const (); 4]>,
}

impl<T> fmt::Pointer for CownPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.cown_ptr.addr(), f)
    }
}

impl<T> std::ops::Drop for CownPtr<T> {
    fn drop(&mut self) {
        unsafe { ffi::boxcar_cownptr_drop(&mut self.cown_ptr) };
    }
}

impl<T> Clone for crate::cown::CownPtr<T> {
    fn clone(&self) -> Self {
        unsafe {
            let mut new = mem::zeroed();
            ffi::boxcar_cownptr_clone(&self.cown_ptr, &mut new);
            Self {
                cown_ptr: new,
                _marker: PhantomData,
            }
        }
    }
}

extern "C" fn drop_glue<T>(cown: *mut ()) {
    let data_ptr = cown_to_data::<T>(cown);
    unsafe {
        ptr::drop_in_place(data_ptr);
    }
}

const SIZEOF_OBJECT_HEADER: usize = 16;
const OBJECT_ALIGNMENT: usize = 16;
const fn vsizeof<T>() -> usize {
    use std::mem::size_of;
    // The runtime stores an object header below the returned pointer, but we still need space for it in the allocation.
    align_up(size_of::<T>() + SIZEOF_OBJECT_HEADER, OBJECT_ALIGNMENT)
}
const fn align_up(value: usize, alignment: usize) -> usize {
    assert!(alignment.is_power_of_two());
    let align_1 = alignment - 1;
    return (value + align_1) & !align_1;
}

impl<T> CownPtr<T> {
    const ALLOCATION_SIZE: usize = vsizeof::<CownDataToxic<T>>();

    /// Must be inside a runtime.
    // TODO: Enforce that.
    pub fn new(value: T) -> Self {
        unsafe {
            // The C++ code called here will read from the old value of cown to attempt to free it.
            // Luckely for us, `nullptr` is a valid value for a cown_ptr, and we can create one easily.
            let mut cown_ptr = mem::zeroed();

            ffi::boxcar_cownptr_new(Self::ALLOCATION_SIZE, drop_glue::<T>, &mut cown_ptr);

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
    fn actualcown_constats_right() {
        let mut sizeof_actualcown = 0;
        let mut alignof_actualcown = 0;
        let mut sizeof_object_header = 0;
        let mut object_alignment = 0;

        unsafe {
            ffi::boxcar_size_info(
                &mut sizeof_actualcown,
                &mut alignof_actualcown,
                &mut sizeof_object_header,
                &mut object_alignment,
            );
        }

        assert_eq!(std::mem::size_of::<ActualCown>(), sizeof_actualcown);
        assert_eq!(std::mem::align_of::<ActualCown>(), alignof_actualcown);

        assert_eq!(sizeof_object_header, SIZEOF_OBJECT_HEADER);
        assert_eq!(object_alignment, OBJECT_ALIGNMENT)
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
