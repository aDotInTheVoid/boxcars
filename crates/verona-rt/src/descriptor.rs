use core::ptr;

use verona_rt_sys::descriptor as ffi;
use verona_rt_sys::descriptor::{Descriptor, Object};
use verona_rt_sys::vsizeof;

use crate::cown::{cown_to_data, CownData};

/// `static Descriptor* desc()` in `vobject.h`
const fn make_desciptor<T>() -> Descriptor {
    let size = vsizeof::<CownData<T>>();

    ffi::Descriptor {
        size,
        trace: noop_trace,
        finaliser: None,
        notified: None,
        destructor: Some(drop_glue_for::<T>),
    }
}

extern "C" fn drop_glue_for<T>(obj: *mut Object) {
    let t_ptr: *mut T = cown_to_data::<T>(obj as _);
    unsafe { ptr::drop_in_place(t_ptr) }
}

extern "C" fn noop_trace(_o: *const ffi::Object, _os: *mut ffi::ObjectStack) {}

// Incredible workaround for static promotion.
trait Hack {
    const DESC: &'static Descriptor;
}
impl<T> Hack for T {
    const DESC: &'static Descriptor = &make_desciptor::<T>();
}
pub(crate) const fn get_desc<T>() -> &'static Descriptor {
    // TODO: Use this when inline const gets stabilized.
    // &const { make_desciptor::<T>() }
    <T as Hack>::DESC
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
