use core::ptr;

use verona_rt_sys::descriptor as ffi;
use verona_rt_sys::descriptor::{Descriptor, Object};
use verona_rt_sys::vsizeof;

/// `static Descriptor* desc()` in `vobject.h`
const fn make_desciptor<T>() -> Descriptor {
    let size = vsizeof::<T>();

    ffi::Descriptor {
        size,
        trace: noop_trace,
        finaliser: None,
        notified: None,
        destructor: Some(drop_glue_for::<T>),
    }
}

extern "C" fn drop_glue_for<T>(obj: *mut Object) {
    unsafe { ptr::drop_in_place::<T>(obj.cast()) }
}

extern "C" fn noop_trace(_o: *const ffi::Object, _os: *mut ffi::ObjectStack) {}

pub(crate) const fn get_desc<T>() -> &'static Descriptor {
    //                 ______
    //           _____/      \\_____
    //          |  _     ___   _   ||
    //          | | \     |   | \  ||
    //          | |  |    |   |  | ||
    //          | |_/     |   |_/  ||
    //          | | \     |   |    ||
    //          | |  \    |   |    ||
    //          | |   \. _|_. | .  ||
    //          |                  ||
    //          |    hack trait    ||
    //          |                  ||
    //  *       | *   **    * **   |**      **
    //   \))ejm97/.,(//,,..,,\||(,,.,\\,.((//
    const { &make_desciptor::<T>() }
}

#[test]
fn descriptor_ptr_eq() {
    let i1 = get_desc::<i32>();
    let i2 = get_desc::<i32>();
    let v1 = get_desc::<Vec<i32>>();
    let v2 = get_desc::<Vec<i32>>();

    assert!(ptr::eq(i1, i2));
    assert!(ptr::eq(v1, v2));
    assert!(!ptr::eq(i1, v1));
}
