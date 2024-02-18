use verona_rt_sys::descriptor::{self as ffi, Object};

use crate::cown::{cown_to_data, vsizeof};

/// `static Descriptor* desc()` in `vobject.h`
const fn make_desciptor<T>() -> ffi::Descriptor {
    let size = vsizeof::<T>();

    ffi::Descriptor {
        size,
        trace: noop_trace,
        finaliser: None,
        notified: None,
        destructor: Some(destructor_for::<T>),
    }
}

extern "C" fn destructor_for<T>(obj: *mut Object) {
    let t_ptr: *mut T = cown_to_data(obj as _);
    unsafe { core::ptr::drop_in_place(t_ptr) }
}

extern "C" fn noop_trace(o: *const ffi::Object, os: *mut ffi::ObjectStack) {}
